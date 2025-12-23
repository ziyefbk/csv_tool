use crate::error::{CsvError, Result};
use crate::csv::{RowIndex, PageCache, IndexMetadata};
use memmap2::{Mmap, MmapOptions};
use std::borrow::Cow;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// CSV文件信息
#[derive(Debug, Clone)]
pub struct CsvInfo {
    /// 文件路径
    pub file_path: PathBuf,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 总行数（不包括表头）
    pub total_rows: usize,
    /// 总列数
    pub total_cols: usize,
    /// 表头
    pub headers: Vec<String>,
}

/// CSV记录（零拷贝）
/// 字段直接引用内存映射的数据，不分配新字符串
#[derive(Debug, Clone)]
pub struct CsvRecord<'a> {
    /// 字段列表（引用mmap数据）
    pub fields: Vec<Cow<'a, str>>,
}

impl<'a> CsvRecord<'a> {
    /// 从字节数据解析一行CSV记录
    /// 
    /// # 参数
    /// - `line`: 一行的字节数据（不包括换行符）
    /// - `delimiter`: 分隔符（默认逗号）
    pub fn parse_line(line: &'a [u8], delimiter: u8) -> Self {
        let mut fields = Vec::new();
        let mut start = 0;
        let mut in_quotes = false;

        for (i, &byte) in line.iter().enumerate() {
            match byte {
                b'"' => {
                    in_quotes = !in_quotes;
                }
                _ if byte == delimiter && !in_quotes => {
                    // 找到分隔符
                    if start < i {
                        let field = &line[start..i];
                        fields.push(Self::parse_field(field));
                    } else {
                        fields.push(Cow::Borrowed(""));
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }

        // 添加最后一个字段
        if start < line.len() {
            let field = &line[start..];
            fields.push(Self::parse_field(field));
        } else {
            fields.push(Cow::Borrowed(""));
        }

        Self { fields }
    }

    /// 解析单个字段（处理引号和转义）
    fn parse_field(field: &[u8]) -> Cow<str> {
        // 移除首尾的引号
        let field = if field.len() >= 2 && field[0] == b'"' && field[field.len() - 1] == b'"' {
            &field[1..field.len() - 1]
        } else {
            field
        };

        // 处理转义的引号（"" -> "）
        if field.contains(&b'"') {
            let mut result = Vec::with_capacity(field.len());
            let mut i = 0;
            while i < field.len() {
                if i < field.len() - 1 && field[i] == b'"' && field[i + 1] == b'"' {
                    result.push(b'"');
                    i += 2;
                } else {
                    result.push(field[i]);
                    i += 1;
                }
            }
            // 转换为owned字符串
            match String::from_utf8(result) {
                Ok(s) => Cow::Owned(s),
                Err(e) => Cow::Owned(String::from_utf8_lossy(e.as_bytes()).to_string()),
            }
        } else {
            // 可以直接使用零拷贝
            match std::str::from_utf8(field) {
                Ok(s) => Cow::Borrowed(s),
                Err(_) => Cow::Owned(String::from_utf8_lossy(field).to_string()),
            }
        }
    }

    /// 转换为owned版本（用于缓存）
    pub fn to_owned(&self) -> CsvRecord<'static> {
        CsvRecord {
            fields: self.fields.iter().map(|f| Cow::Owned(f.to_string())).collect(),
        }
    }
}

/// 高性能CSV读取器
/// 使用内存映射、行索引和页面缓存
pub struct CsvReader {
    /// 内存映射的文件
    mmap: Arc<Mmap>,
    /// 行索引
    index: RowIndex,
    /// 页面缓存
    cache: PageCache,
    /// 文件信息
    info: CsvInfo,
    /// CSV分隔符
    delimiter: u8,
    /// 数据起始偏移量（跳过表头后的位置）
    data_start_offset: u64,
}

impl CsvReader {
    /// 打开CSV文件并创建读取器
    /// 
    /// # 参数
    /// - `path`: CSV文件路径
    /// - `has_headers`: 是否有表头
    /// - `delimiter`: 分隔符（默认逗号）
    /// - `index_granularity`: 索引粒度（每N行记录一次，默认1000）
    pub fn open<P: AsRef<Path>>(
        path: P,
        has_headers: bool,
        delimiter: u8,
        index_granularity: usize,
    ) -> Result<Self> {
        let path = path.as_ref();
        
        // 获取文件元数据
        let file_metadata = std::fs::metadata(path)?;
        let file_size = file_metadata.len();
        let file_mtime = file_metadata.modified().unwrap_or_else(|_| SystemTime::now());

        // 打开文件并创建内存映射
        let file = File::open(path)?;
        let mmap = Arc::new(
            unsafe { MmapOptions::new().map(&file) }
                .map_err(|e| CsvError::Mmap(e.to_string()))?
        );

        // 读取表头
        let headers = if has_headers {
            Self::read_headers(&mmap, delimiter)?
        } else {
            Vec::new()
        };

        let total_cols = if has_headers {
            headers.len()
        } else {
            // 从第一行推断列数
            Self::count_columns_first_line(&mmap, delimiter)?
        };

        // 尝试加载索引，如果失败则构建新索引
        let (index, total_rows) = Self::load_or_build_index(
            path,
            &mmap,
            has_headers,
            index_granularity,
            file_size,
            file_mtime,
        )?;

        // 计算数据起始偏移量（跳过表头）
        let data_start_offset = if has_headers {
            let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
                3
            } else {
                0
            };
            // 找到第一个换行符后的位置
            let mut offset = start as u64;
            for (i, &byte) in mmap[start..].iter().enumerate() {
                if byte == b'\n' {
                    offset = (start + i + 1) as u64;
                    break;
                }
            }
            offset
        } else {
            if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
                3
            } else {
                0
            }
        };

        let info = CsvInfo {
            file_path: path.to_path_buf(),
            file_size,
            total_rows,
            total_cols,
            headers,
        };

        Ok(Self {
            mmap,
            index,
            cache: PageCache::default(),
            info,
            delimiter,
            data_start_offset,
        })
    }

    /// 读取表头
    fn read_headers(mmap: &Mmap, delimiter: u8) -> Result<Vec<String>> {
        // 跳过BOM
        let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3
        } else {
            0
        };

        // 找到第一行的结束位置
        let mut line_end = start;
        for (i, &byte) in mmap[start..].iter().enumerate() {
            if byte == b'\n' || byte == b'\r' {
                line_end = start + i;
                break;
            }
        }

        if line_end == start {
            return Err(CsvError::Format("文件为空或格式错误".to_string()));
        }

        let header_line = &mmap[start..line_end];
        let record = CsvRecord::parse_line(header_line, delimiter);
        
        Ok(record.fields.iter().map(|f| f.to_string()).collect())
    }

    /// 从第一行推断列数
    fn count_columns_first_line(mmap: &Mmap, delimiter: u8) -> Result<usize> {
        let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3
        } else {
            0
        };

        let mut line_end = start;
        for (i, &byte) in mmap[start..].iter().enumerate() {
            if byte == b'\n' || byte == b'\r' {
                line_end = start + i;
                break;
            }
        }

        if line_end == start {
            return Err(CsvError::Format("文件为空或格式错误".to_string()));
        }

        let first_line = &mmap[start..line_end];
        let record = CsvRecord::parse_line(first_line, delimiter);
        Ok(record.fields.len())
    }

    /// 读取指定页的数据
    /// 
    /// # 参数
    /// - `page`: 页码（从0开始）
    /// - `page_size`: 每页行数
    /// 
    /// # 返回
    /// 该页的记录列表
    pub fn read_page(&mut self, page: usize, page_size: usize) -> Result<Vec<CsvRecord>> {
        // 计算目标行范围
        let start_row = page * page_size;
        let end_row = (start_row + page_size).min(self.info.total_rows);

        if start_row >= self.info.total_rows {
            return Ok(Vec::new());
        }

        // 使用索引快速定位到起始行附近
        let index_offset = self.index.seek_to_row(start_row)? as usize;
        
        // 从起始偏移量开始解析行
        let mut records = Vec::new();
        // 确保从数据区域开始（跳过表头）
        let mut current_offset = index_offset.max(self.data_start_offset as usize);
        let mut current_row = start_row;

        // 如果使用索引定位，需要找到实际的行起始位置
        // 从索引点开始向前找到行首（最多向前查找1000字节）
        if current_offset > 0 {
            let search_start = current_offset.saturating_sub(1000);
            for i in (search_start..current_offset).rev() {
                if self.mmap[i] == b'\n' {
                    current_offset = i + 1;
                    break;
                }
            }
        }

        // 从当前位置开始扫描到目标行
        // 由于索引是稀疏的，我们需要从索引点继续扫描到目标行
        while current_row < start_row && current_offset < self.mmap.len() {
            for (i, &byte) in self.mmap[current_offset..].iter().enumerate() {
                if byte == b'\n' {
                    current_offset += i + 1;
                    current_row += 1;
                    break;
                }
            }
            if current_row >= start_row {
                break;
            }
        }

        // 解析行直到达到目标数量或文件结束
        while current_row < end_row && current_offset < self.mmap.len() {
            // 找到当前行的结束位置
            let mut line_end = current_offset;
            let mut found = false;
            for (i, &byte) in self.mmap[current_offset..].iter().enumerate() {
                if byte == b'\n' {
                    line_end = current_offset + i;
                    found = true;
                    break;
                }
            }

            if !found {
                break; // 文件结束
            }

            // 解析当前行
            let line = &self.mmap[current_offset..line_end];
            let record = CsvRecord::parse_line(line, self.delimiter);
            records.push(record);

            // 移动到下一行
            current_offset = line_end + 1;
            current_row += 1;
        }

        // 存入缓存（转换为owned版本，用于后续快速访问）
        let cached_records: Vec<CsvRecord<'static>> = records.iter()
            .map(|r| r.to_owned())
            .collect();
        self.cache.put(page, cached_records);

        Ok(records)
    }

    /// 获取文件信息
    pub fn info(&self) -> &CsvInfo {
        &self.info
    }

    /// 获取总页数
    pub fn total_pages(&self, page_size: usize) -> usize {
        (self.info.total_rows + page_size - 1) / page_size
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// 加载或构建索引
    /// 
    /// 优先尝试加载已保存的索引，如果索引不存在或无效，则构建新索引并保存
    fn load_or_build_index(
        csv_path: &Path,
        mmap: &Mmap,
        has_headers: bool,
        index_granularity: usize,
        file_size: u64,
        file_mtime: SystemTime,
    ) -> Result<(RowIndex, usize)> {
        let index_path = RowIndex::index_file_path(csv_path);
        
        // 尝试加载索引
        if index_path.exists() {
            match RowIndex::load_from_file(&index_path) {
                Ok((index, metadata)) => {
                    // 验证索引有效性
                    if RowIndex::is_index_valid(csv_path, &metadata) {
                        // 验证索引粒度是否匹配
                        if metadata.granularity == index_granularity {
                            let total_rows = index.total_rows();
                            return Ok((index, total_rows));
                        }
                    }
                    // 索引无效，继续构建新索引
                }
                Err(_) => {
                    // 加载失败，继续构建新索引
                }
            }
        }

        // 构建新索引
        let index = RowIndex::build(mmap, has_headers, index_granularity)?;
        let total_rows = index.total_rows();

        // 保存索引
        let metadata = IndexMetadata::new(
            csv_path.to_path_buf(),
            file_size,
            file_mtime,
            index_granularity,
        );
        
        // 克隆index用于保存，因为save_to_file需要&self，但我们需要返回原始index
        let index_clone = index.clone();
        if let Err(e) = index_clone.save_to_file(csv_path, &metadata) {
            // 索引保存失败不影响使用，只记录警告
            eprintln!("警告: 无法保存索引文件: {}", e);
        }

        Ok((index, total_rows))
    }
}

