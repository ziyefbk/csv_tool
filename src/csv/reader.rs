use crate::error::{CsvError, Result};
use crate::csv::{RowIndex, PageCache, IndexMetadata, RowEstimate};
use memmap2::{Mmap, MmapOptions};
use memchr::memchr;  // SIMD加速的换行符查找
use std::borrow::Cow;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::SystemTime;
use std::thread;

/// 后台索引构建句柄
pub struct IndexBuildHandle {
    handle: Option<thread::JoinHandle<(RowIndex, bool)>>,
    cancel_flag: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
    total_bytes: usize,
}

impl IndexBuildHandle {
    /// 获取构建进度（0.0 - 100.0）
    pub fn progress(&self) -> f64 {
        let current = self.progress.load(Ordering::Relaxed);
        if self.total_bytes == 0 {
            100.0
        } else {
            (current as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// 取消构建
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    /// 等待构建完成并返回结果
    pub fn wait(mut self) -> Option<(RowIndex, bool)> {
        self.handle.take().and_then(|h| h.join().ok())
    }

    /// 检查是否完成
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(true)
    }
}

impl Drop for IndexBuildHandle {
    fn drop(&mut self) {
        // 如果句柄被丢弃但线程还在运行，设置取消标志
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

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
        // 去除行尾的 \r（处理 Windows 换行符 CRLF）
        let line = if !line.is_empty() && line[line.len() - 1] == b'\r' {
            &line[..line.len() - 1]
        } else {
            line
        };
        
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
    fn parse_field(field: &[u8]) -> Cow<'_, str> {
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
    /// 是否有表头
    has_headers: bool,
    /// 索引粒度
    index_granularity: usize,
    /// 后台索引构建取消标志
    cancel_flag: Arc<AtomicBool>,
    /// 后台索引构建进度
    build_progress: Arc<AtomicUsize>,
    /// 行数估算（如果尚未完成精确计数）
    row_estimate: Option<RowEstimate>,
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

        // 计算数据起始偏移量（跳过表头）- 使用memchr加速
        let data_start_offset = if has_headers {
            let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
                3
            } else {
                0
            };
            // 找到第一个换行符后的位置
            let header_slice = &mmap[start..];
            if let Some(pos) = memchr(b'\n', header_slice) {
                (start + pos + 1) as u64
            } else {
                start as u64
            }
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
            has_headers,
            index_granularity,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            build_progress: Arc::new(AtomicUsize::new(0)),
            row_estimate: None,
        })
    }

    /// 快速打开CSV文件（毫秒级响应）
    /// 
    /// 与普通 `open` 不同，此方法：
    /// 1. 使用采样估算行数，而不是扫描整个文件
    /// 2. 只构建前几页的索引
    /// 3. 可以在后台继续构建完整索引
    /// 
    /// # 参数
    /// - `path`: CSV文件路径
    /// - `has_headers`: 是否有表头
    /// - `delimiter`: 分隔符
    /// - `index_granularity`: 索引粒度
    /// 
    /// # 性能
    /// 对于任意大小的文件，都能在 100ms 以内返回
    pub fn open_fast<P: AsRef<Path>>(
        path: P,
        has_headers: bool,
        delimiter: u8,
        index_granularity: usize,
    ) -> Result<Self> {
        let path = path.as_ref();
        
        // 获取文件元数据
        let file_metadata = std::fs::metadata(path)?;
        let file_size = file_metadata.len();

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
            Self::count_columns_first_line(&mmap, delimiter)?
        };

        // 尝试加载已有索引
        let index_path = RowIndex::index_file_path(path);
        let (index, total_rows, row_estimate) = if index_path.exists() {
            match RowIndex::load_from_file(&index_path) {
                Ok((index, metadata)) => {
                    if RowIndex::is_index_valid(path, &metadata) && metadata.granularity == index_granularity {
                        let total_rows = index.total_rows();
                        (index, total_rows, None)
                    } else {
                        // 索引无效，使用快速模式
                        Self::build_fast_index(&mmap, has_headers, index_granularity)?
                    }
                }
                Err(_) => Self::build_fast_index(&mmap, has_headers, index_granularity)?,
            }
        } else {
            Self::build_fast_index(&mmap, has_headers, index_granularity)?
        };

        // 计算数据起始偏移量
        let data_start_offset = if has_headers {
            let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" { 3 } else { 0 };
            let header_slice = &mmap[start..];
            if let Some(pos) = memchr(b'\n', header_slice) {
                (start + pos + 1) as u64
            } else {
                start as u64
            }
        } else {
            if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" { 3 } else { 0 }
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
            has_headers,
            index_granularity,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            build_progress: Arc::new(AtomicUsize::new(0)),
            row_estimate,
        })
    }

    /// 快速构建索引（采样估算 + 部分索引）
    /// 
    /// 使用更激进的优化策略：
    /// - 智能采样大小（根据文件大小调整）
    /// - 最小初始索引（只索引前 500 行）
    fn build_fast_index(
        mmap: &Mmap,
        has_headers: bool,
        granularity: usize,
    ) -> Result<(RowIndex, usize, Option<RowEstimate>)> {
        let file_size = mmap.len();
        
        // 智能采样策略：根据文件大小调整采样大小
        // - 小文件 (<10MB): 256KB
        // - 中文件 (10-100MB): 128KB  
        // - 大文件 (>100MB): 64KB
        const SMALL_FILE_THRESHOLD: usize = 10 * 1024 * 1024;
        const MEDIUM_FILE_THRESHOLD: usize = 100 * 1024 * 1024;
        
        let sample_size = if file_size < SMALL_FILE_THRESHOLD {
            256 * 1024  // 256KB for small files
        } else if file_size < MEDIUM_FILE_THRESHOLD {
            128 * 1024  // 128KB for medium files
        } else {
            64 * 1024   // 64KB for large files (>100MB)
        };
        
        let estimate = RowIndex::estimate_rows(mmap, has_headers, sample_size);
        
        // 对于小文件（<1MB），直接构建完整索引（通常 <100ms）
        const TINY_FILE_THRESHOLD: usize = 1 * 1024 * 1024;
        if file_size <= TINY_FILE_THRESHOLD || estimate.is_exact {
            let index = RowIndex::build(mmap, has_headers, granularity)?;
            let total_rows = index.total_rows();
            return Ok((index, total_rows, None));
        }

        // 对于大文件，只构建前 500 行的索引（确保首页立即可用）
        // 从 2000 行降低到 500 行，进一步提升打开速度
        const INITIAL_ROWS: usize = 500;
        let (index, _complete) = RowIndex::build_partial(mmap, has_headers, granularity, Some(INITIAL_ROWS))?;
        
        // 使用估算的行数（但至少是已索引的行数）
        let total_rows = estimate.estimated_rows.max(index.total_rows());
        
        Ok((index, total_rows, Some(estimate)))
    }

    /// 在后台继续构建完整索引
    /// 
    /// # 返回
    /// 返回一个句柄，可以用于等待构建完成或取消构建
    pub fn build_index_async(&mut self) -> IndexBuildHandle {
        let mmap = Arc::clone(&self.mmap);
        let mut index = self.index.clone();
        let cancel_flag = Arc::clone(&self.cancel_flag);
        let progress = Arc::clone(&self.build_progress);
        let granularity = self.index_granularity;
        let _has_headers = self.has_headers; // 保留用于未来扩展
        let file_path = self.info.file_path.clone();
        let file_size = self.info.file_size;
        let file_mtime = std::fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        let handle = thread::spawn(move || {
            // 继续构建索引
            let result = index.continue_build(&mmap, Some(&cancel_flag), Some(&progress));
            
            if let Ok(true) = result {
                // 索引构建完成，保存到文件
                let metadata = IndexMetadata::new(
                    file_path.clone(),
                    file_size,
                    file_mtime,
                    granularity,
                );
                let _ = index.save_to_file(&file_path, &metadata);
            }

            (index, result.is_ok())
        });

        IndexBuildHandle {
            handle: Some(handle),
            cancel_flag: Arc::clone(&self.cancel_flag),
            progress: Arc::clone(&self.build_progress),
            total_bytes: self.info.file_size as usize,
        }
    }

    /// 更新索引（从后台构建结果）
    pub fn update_index(&mut self, new_index: RowIndex) {
        self.info.total_rows = new_index.total_rows();
        self.index = new_index;
        self.row_estimate = None; // 清除估算值，使用精确值
        self.cache.clear(); // 清除缓存，因为行数可能变化
    }

    /// 检查索引是否完成
    pub fn is_index_complete(&self) -> bool {
        self.index.is_complete()
    }

    /// 获取行数估算信息（如果有）
    pub fn row_estimate(&self) -> Option<&RowEstimate> {
        self.row_estimate.as_ref()
    }

    /// 获取索引构建进度（0-100）
    pub fn index_build_progress(&self) -> f64 {
        let progress = self.build_progress.load(Ordering::Relaxed);
        let total = self.info.file_size as usize;
        if total == 0 {
            100.0
        } else {
            (progress as f64 / total as f64) * 100.0
        }
    }

    /// 读取表头
    fn read_headers(mmap: &Mmap, delimiter: u8) -> Result<Vec<String>> {
        // 跳过BOM
        let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3
        } else {
            0
        };

        // 找到第一行的结束位置 - 使用memchr加速
        let header_slice = &mmap[start..];
        let line_end = memchr(b'\n', header_slice)
            .or_else(|| memchr(b'\r', header_slice))
            .map(|pos| start + pos)
            .unwrap_or(mmap.len());

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

        // 找到第一行的结束位置 - 使用memchr加速
        let first_slice = &mmap[start..];
        let line_end = memchr(b'\n', first_slice)
            .or_else(|| memchr(b'\r', first_slice))
            .map(|pos| start + pos)
            .unwrap_or(mmap.len());

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
    pub fn read_page(&mut self, page: usize, page_size: usize) -> Result<Vec<CsvRecord<'_>>> {
        // 计算目标行范围
        let start_row = page * page_size;
        let end_row = (start_row + page_size).min(self.info.total_rows);

        if start_row >= self.info.total_rows {
            return Ok(Vec::new());
        }

        // 使用索引快速定位到起始行附近
        let (index_offset, index_row) = self.index.seek_to_row_with_info(start_row)?;
        let index_offset = index_offset as usize;
        
        // 从起始偏移量开始解析行
        let mut records = Vec::new();
        // 确保从数据区域开始（跳过表头）
        let mut current_offset = index_offset.max(self.data_start_offset as usize);
        // 设置当前行号为索引点对应的行号，如果从数据开头开始则为0
        let mut current_row = if index_offset <= self.data_start_offset as usize {
            0
        } else {
            index_row
        };

        // 如果使用索引定位，需要找到实际的行起始位置
        // 从索引点开始向前找到行首（最多向前查找1000字节）
        if current_offset > 0 && current_offset > self.data_start_offset as usize {
            let search_start = current_offset.saturating_sub(1000);
            for i in (search_start..current_offset).rev() {
                if self.mmap[i] == b'\n' {
                    current_offset = i + 1;
                    break;
                }
            }
        }

        // 从当前位置开始扫描到目标行 - 使用memchr加速
        // 由于索引是稀疏的，我们需要从索引点继续扫描到目标行
        while current_row < start_row && current_offset < self.mmap.len() {
            let remaining = &self.mmap[current_offset..];
            if let Some(pos) = memchr(b'\n', remaining) {
                current_offset += pos + 1;
                current_row += 1;
            } else {
                break; // 文件结束
            }
            if current_row >= start_row {
                break;
            }
        }

        // 解析行直到达到目标数量或文件结束 - 使用memchr加速
        while current_row < end_row && current_offset < self.mmap.len() {
            // 找到当前行的结束位置
            let remaining = &self.mmap[current_offset..];
            let line_end = if let Some(pos) = memchr(b'\n', remaining) {
                current_offset + pos
            } else {
                // 文件结束，但可能还有最后一行
                if current_offset < self.mmap.len() {
                    self.mmap.len()
                } else {
                    break; // 文件结束
                }
            };

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

    /// 搜索CSV文件
    /// 
    /// # 参数
    /// - `options`: 搜索选项
    /// 
    /// # 返回
    /// 搜索结果列表
    pub fn search(&self, options: &crate::csv::search::SearchOptions) -> Result<Vec<crate::csv::search::SearchResult>> {
        use crate::csv::search::{Searcher, SearchResult};
        
        let searcher = Searcher::new(options.clone());
        let mut results = Vec::new();
        let max_results = options.max_results.unwrap_or(usize::MAX);
        
        // 从数据起始位置开始扫描
        let mut current_offset = self.data_start_offset as usize;
        let mut row_number = 0;
        
        while current_offset < self.mmap.len() && results.len() < max_results {
            // 找到当前行的结束位置 - 使用memchr加速
            let remaining = &self.mmap[current_offset..];
            let line_end = if let Some(pos) = memchr(b'\n', remaining) {
                current_offset + pos
            } else {
                // 文件结束，但可能还有最后一行
                if current_offset < self.mmap.len() {
                    self.mmap.len()
                } else {
                    break;
                }
            };
            
            // 解析当前行
            let line = &self.mmap[current_offset..line_end];
            let record = CsvRecord::parse_line(line, self.delimiter);
            
            // 检查是否匹配
            if let Some(matches) = searcher.matches_record(&record) {
                results.push(SearchResult {
                    row_number,
                    matches,
                    record: record.to_owned(),
                });
            }
            
            // 移动到下一行
            current_offset = line_end + 1;
            row_number += 1;
        }
        
        Ok(results)
    }

    /// 统计匹配数量（不返回详细结果，更高效）
    pub fn count_matches(&self, options: &crate::csv::search::SearchOptions) -> Result<usize> {
        use crate::csv::search::Searcher;
        
        let searcher = Searcher::new(options.clone());
        let mut count = 0;
        
        // 从数据起始位置开始扫描
        let mut current_offset = self.data_start_offset as usize;
        
        while current_offset < self.mmap.len() {
            // 找到当前行的结束位置 - 使用memchr加速
            let remaining = &self.mmap[current_offset..];
            let line_end = if let Some(pos) = memchr(b'\n', remaining) {
                current_offset + pos
            } else {
                // 文件结束，但可能还有最后一行
                if current_offset < self.mmap.len() {
                    self.mmap.len()
                } else {
                    break;
                }
            };
            
            // 解析并检查匹配
            let line = &self.mmap[current_offset..line_end];
            let record = CsvRecord::parse_line(line, self.delimiter);
            
            if searcher.is_match(&record) {
                count += 1;
            }
            
            current_offset = line_end + 1;
        }
        
        Ok(count)
    }

    /// 获取表头
    pub fn headers(&self) -> &[String] {
        &self.info.headers
    }

    /// 获取分隔符
    pub fn delimiter(&self) -> u8 {
        self.delimiter
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

        // 构建新索引（这里不传递进度回调，因为调用者会处理）
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

