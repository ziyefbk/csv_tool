use crate::error::{CsvError, Result};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// 索引元数据
/// 用于验证索引的有效性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// CSV文件路径
    pub csv_path: PathBuf,
    /// CSV文件大小（字节）
    pub csv_size: u64,
    /// CSV文件修改时间
    pub csv_mtime: SystemTime,
    /// 索引格式版本
    pub index_version: u32,
    /// 索引构建时间
    pub build_time: SystemTime,
    /// 索引粒度
    pub granularity: usize,
}

impl IndexMetadata {
    /// 创建新的索引元数据
    pub fn new(csv_path: PathBuf, csv_size: u64, csv_mtime: SystemTime, granularity: usize) -> Self {
        Self {
            csv_path,
            csv_size,
            csv_mtime,
            index_version: 1, // 当前索引格式版本
            build_time: SystemTime::now(),
            granularity,
        }
    }
}

/// 稀疏行索引结构
/// 每N行记录一次字节偏移，用于快速定位到目标行附近
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowIndex {
    /// 字节偏移量列表
    offsets: Vec<u64>,
    /// 对应的行号列表（不包括表头）
    row_numbers: Vec<usize>,
    /// 索引粒度（每N行记录一次）
    granularity: usize,
    /// 总行数（不包括表头）
    total_rows: usize,
}

impl RowIndex {
    /// 创建新的索引
    pub fn new(granularity: usize) -> Self {
        Self {
            offsets: Vec::new(),
            row_numbers: Vec::new(),
            granularity,
            total_rows: 0,
        }
    }

    /// 从内存映射文件构建索引
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `granularity`: 索引粒度（每N行记录一次）
    pub fn build(mmap: &Mmap, has_headers: bool, granularity: usize) -> Result<Self> {
        let mut offsets = Vec::new();
        let mut row_numbers = Vec::new();
        let mut current_row = 0;
        let mut current_offset = 0u64;
        let mut line_start = 0u64;

        // 如果有多余的字节，跳过BOM标记
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3u64
        } else {
            0u64
        };

        // 跳过表头（如果存在）
        if has_headers {
            for (i, &byte) in mmap[start_offset as usize..].iter().enumerate() {
                if byte == b'\n' {
                    line_start = start_offset + i as u64 + 1;
                    current_offset = line_start;
                    break;
                }
            }
        } else {
            current_offset = start_offset;
            line_start = start_offset;
        }

        // 扫描文件，记录索引点
        let mut last_newline_pos = None;
        for (i, &byte) in mmap[current_offset as usize..].iter().enumerate() {
            if byte == b'\n' {
                last_newline_pos = Some(current_offset + i as u64);
                current_row += 1;
                
                // 每N行记录一次索引点
                if current_row % granularity == 0 {
                    offsets.push(line_start);
                    row_numbers.push(current_row);
                }
                
                // 更新下一行的起始位置
                line_start = current_offset + i as u64 + 1;
            }
        }

        // 处理最后一行（如果文件末尾没有换行符，但还有内容）
        if let Some(last_nl) = last_newline_pos {
            // 如果最后一个换行符之后还有内容，说明还有一行
            if ((last_nl + 1) as usize) < mmap.len() {
                current_row += 1;
            }
        } else {
            // 如果没有找到任何换行符，但文件不为空，说明只有一行
            if (current_offset as usize) < mmap.len() {
                current_row = 1;
            }
        }

        Ok(Self {
            offsets,
            row_numbers,
            granularity,
            total_rows: current_row,
        })
    }

    /// 查找目标行对应的字节偏移量
    /// 
    /// # 参数
    /// - `target_row`: 目标行号（不包括表头，从0开始）
    /// 
    /// # 返回
    /// 字节偏移量，用于定位到目标行附近
    pub fn seek_to_row(&self, target_row: usize) -> Result<u64> {
        let (offset, _) = self.seek_to_row_with_info(target_row)?;
        Ok(offset)
    }

    /// 查找目标行对应的字节偏移量和索引点行号
    /// 
    /// # 参数
    /// - `target_row`: 目标行号（不包括表头，从0开始）
    /// 
    /// # 返回
    /// (字节偏移量, 索引点对应的行号)
    /// 如果没有合适的索引点（目标行在第一个索引点之前），返回 (0, 0)
    pub fn seek_to_row_with_info(&self, target_row: usize) -> Result<(u64, usize)> {
        if target_row >= self.total_rows {
            return Err(CsvError::IndexOutOfBounds {
                row: target_row,
                total_rows: self.total_rows,
            });
        }

        // 如果索引为空，从头开始
        if self.offsets.is_empty() {
            return Ok((0, 0));
        }

        // 如果目标行在第一个索引点之前，从头开始
        if target_row < self.row_numbers[0] {
            return Ok((0, 0));
        }

        // 二分查找找到最近的索引点
        let idx = match self.row_numbers.binary_search(&target_row) {
            Ok(i) => i, // 精确匹配
            Err(i) => {
                // 找到插入位置，使用前一个索引点
                // 这里 i > 0 因为我们已经处理了 target_row < row_numbers[0] 的情况
                i - 1
            }
        };

        Ok((self.offsets[idx], self.row_numbers[idx]))
    }

    /// 获取总行数
    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    /// 获取索引粒度
    pub fn granularity(&self) -> usize {
        self.granularity
    }

    /// 获取索引点数量
    pub fn index_count(&self) -> usize {
        self.offsets.len()
    }

    /// 生成索引文件路径
    /// 
    /// # 参数
    /// - `csv_path`: CSV文件路径
    /// 
    /// # 返回
    /// 索引文件路径（在CSV文件同目录下，文件名后加.idx）
    pub fn index_file_path(csv_path: &Path) -> PathBuf {
        let mut path = csv_path.to_path_buf();
        // 获取原始扩展名，如果没有则使用"csv"
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("csv");
        path.set_extension(format!("{}.idx", ext));
        path
    }

    /// 保存索引到文件
    /// 
    /// # 参数
    /// - `csv_path`: CSV文件路径
    /// - `metadata`: 索引元数据
    /// 
    /// # 返回
    /// 成功时返回索引文件路径
    pub fn save_to_file(&self, csv_path: &Path, metadata: &IndexMetadata) -> Result<PathBuf> {
        let index_path = Self::index_file_path(csv_path);
        
        let mut file = File::create(&index_path)
            .map_err(|e| CsvError::IndexFile(format!("无法创建索引文件: {}", e)))?;

        // 序列化元数据
        let metadata_bytes = bincode::serialize(metadata)
            .map_err(|e| CsvError::IndexFile(format!("序列化元数据失败: {}", e)))?;
        
        // 序列化索引
        let index_bytes = bincode::serialize(self)
            .map_err(|e| CsvError::IndexFile(format!("序列化索引失败: {}", e)))?;

        // 写入文件格式：
        // [元数据长度: u64][元数据][索引数据]
        let metadata_len = metadata_bytes.len() as u64;
        file.write_all(&metadata_len.to_le_bytes())
            .map_err(|e| CsvError::IndexFile(format!("写入元数据长度失败: {}", e)))?;
        file.write_all(&metadata_bytes)
            .map_err(|e| CsvError::IndexFile(format!("写入元数据失败: {}", e)))?;
        file.write_all(&index_bytes)
            .map_err(|e| CsvError::IndexFile(format!("写入索引数据失败: {}", e)))?;

        Ok(index_path)
    }

    /// 从文件加载索引
    /// 
    /// # 参数
    /// - `index_path`: 索引文件路径
    /// 
    /// # 返回
    /// 成功时返回(索引, 元数据)
    pub fn load_from_file(index_path: &Path) -> Result<(Self, IndexMetadata)> {
        let mut file = File::open(index_path)
            .map_err(|e| CsvError::IndexFile(format!("无法打开索引文件: {}", e)))?;

        // 读取元数据长度
        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)
            .map_err(|e| CsvError::IndexFile(format!("读取元数据长度失败: {}", e)))?;
        let metadata_len = u64::from_le_bytes(len_bytes) as usize;

        // 读取元数据
        let mut metadata_bytes = vec![0u8; metadata_len];
        file.read_exact(&mut metadata_bytes)
            .map_err(|e| CsvError::IndexFile(format!("读取元数据失败: {}", e)))?;
        
        let metadata: IndexMetadata = bincode::deserialize(&metadata_bytes)
            .map_err(|e| CsvError::IndexFile(format!("反序列化元数据失败: {}", e)))?;

        // 读取索引数据（剩余所有数据）
        let mut index_bytes = Vec::new();
        file.read_to_end(&mut index_bytes)
            .map_err(|e| CsvError::IndexFile(format!("读取索引数据失败: {}", e)))?;

        let index: RowIndex = bincode::deserialize(&index_bytes)
            .map_err(|e| CsvError::IndexFile(format!("反序列化索引失败: {}", e)))?;

        Ok((index, metadata))
    }

    /// 验证索引是否有效
    /// 
    /// # 参数
    /// - `csv_path`: CSV文件路径
    /// - `metadata`: 索引元数据
    /// 
    /// # 返回
    /// 如果索引有效返回true，否则返回false
    pub fn is_index_valid(csv_path: &Path, metadata: &IndexMetadata) -> bool {
        // 检查文件是否存在
        if !csv_path.exists() {
            return false;
        }

        // 检查路径是否匹配（规范化路径比较）
        let csv_path_normalized = csv_path.canonicalize().ok();
        let metadata_path_normalized = metadata.csv_path.canonicalize().ok();
        
        if let (Some(csv), Some(meta)) = (csv_path_normalized, metadata_path_normalized) {
            if csv != meta {
                return false;
            }
        }

        // 检查文件大小
        if let Ok(metadata_file) = std::fs::metadata(csv_path) {
            if metadata_file.len() != metadata.csv_size {
                return false;
            }

            // 检查文件修改时间（允许1秒误差，因为文件系统精度问题）
            if let Ok(mtime) = metadata_file.modified() {
                let time_diff = mtime.duration_since(metadata.csv_mtime)
                    .or_else(|_| metadata.csv_mtime.duration_since(mtime))
                    .ok();
                
                if let Some(diff) = time_diff {
                    // 如果时间差超过1秒，认为文件已修改
                    if diff.as_secs() > 1 {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }

        // 检查索引版本兼容性
        if metadata.index_version != 1 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use memmap2::MmapOptions;

    #[test]
    fn test_build_index() {
        // 创建测试CSV内容（3行数据 + 1行表头）
        let content = b"header1,header2\nrow1,col1\nrow2,col2\nrow3,col3\n";
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_csv_index.csv");
        std::fs::write(&temp_file, content).unwrap();

        let file = File::open(&temp_file).unwrap();
        let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

        let index = RowIndex::build(&mmap, true, 1).unwrap();
        // 应该有3行数据（不包括表头）
        assert_eq!(index.total_rows(), 3);
        // 索引粒度是1，所以应该有3个索引点（每行一个）
        assert_eq!(index.index_count(), 3);
        
        // 测试跳转功能
        let offset = index.seek_to_row(1).unwrap();
        assert!(offset > 0);
        
        // 清理
        let _ = std::fs::remove_file(&temp_file);
    }
}

