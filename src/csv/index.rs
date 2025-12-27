use crate::error::{CsvError, Result};
use memmap2::Mmap;
use memchr::memchr_iter;  // SIMD加速的换行符查找
use rayon::prelude::*;  // 并行处理
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

/// 行数估算结果
#[derive(Debug, Clone)]
pub struct RowEstimate {
    /// 估算的总行数
    pub estimated_rows: usize,
    /// 是否是精确值（已完成完整扫描）
    pub is_exact: bool,
    /// 采样的字节数
    pub sampled_bytes: usize,
    /// 文件总字节数
    pub total_bytes: usize,
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
    /// 是否已完成完整索引构建
    #[serde(default = "default_true")]
    is_complete: bool,
    /// 已索引的字节偏移量（用于增量构建）
    #[serde(default)]
    indexed_bytes: u64,
}

fn default_true() -> bool {
    true
}

impl RowIndex {
    /// 创建新的索引
    pub fn new(granularity: usize) -> Self {
        Self {
            offsets: Vec::new(),
            row_numbers: Vec::new(),
            granularity,
            total_rows: 0,
            is_complete: false,
            indexed_bytes: 0,
        }
    }

    /// 快速采样估算行数（不扫描整个文件）
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `sample_size`: 采样大小（字节），默认采样前 1MB
    /// 
    /// # 性能
    /// 对于任意大小的文件，都能在毫秒级完成估算
    pub fn estimate_rows(mmap: &Mmap, has_headers: bool, sample_size: usize) -> RowEstimate {
        let total_bytes = mmap.len();
        
        // 如果文件很小，直接精确计数
        if total_bytes <= sample_size {
            let exact_count = Self::count_rows_exact(mmap, has_headers);
            return RowEstimate {
                estimated_rows: exact_count,
                is_exact: true,
                sampled_bytes: total_bytes,
                total_bytes,
            };
        }

        // 计算数据起始位置（跳过BOM和表头）
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3usize
        } else {
            0usize
        };

        let data_start = if has_headers {
            let header_slice = &mmap[start_offset..];
            if let Some(pos) = memchr::memchr(b'\n', header_slice) {
                start_offset + pos + 1
            } else {
                start_offset
            }
        } else {
            start_offset
        };

        // 采样区域
        let sample_end = (data_start + sample_size).min(total_bytes);
        let sample_slice = &mmap[data_start..sample_end];
        
        // 计算采样区域的行数
        let sample_rows = memchr_iter(b'\n', sample_slice).count();
        let sampled_bytes = sample_end - data_start;

        // 如果采样区域没有换行符，假设整个文件就是一行
        if sample_rows == 0 {
            return RowEstimate {
                estimated_rows: 1,
                is_exact: false,
                sampled_bytes,
                total_bytes,
            };
        }

        // 计算平均每行字节数，然后估算总行数
        let bytes_per_row = sampled_bytes as f64 / sample_rows as f64;
        let data_bytes = total_bytes - data_start;
        let estimated_rows = (data_bytes as f64 / bytes_per_row).ceil() as usize;

        RowEstimate {
            estimated_rows,
            is_exact: false,
            sampled_bytes,
            total_bytes,
        }
    }

    /// 精确计算行数（扫描整个文件）
    fn count_rows_exact(mmap: &Mmap, has_headers: bool) -> usize {
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3usize
        } else {
            0usize
        };

        let data_start = if has_headers {
            let header_slice = &mmap[start_offset..];
            if let Some(pos) = memchr::memchr(b'\n', header_slice) {
                start_offset + pos + 1
            } else {
                return 0;
            }
        } else {
            start_offset
        };

        if data_start >= mmap.len() {
            return 0;
        }

        let data_slice = &mmap[data_start..];
        let newline_count = memchr_iter(b'\n', data_slice).count();

        // 如果文件最后没有换行符但有内容，加1
        if !data_slice.is_empty() && data_slice[data_slice.len() - 1] != b'\n' {
            newline_count + 1
        } else {
            newline_count
        }
    }

    /// 构建部分索引（只索引前N行）
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `granularity`: 索引粒度
    /// - `max_rows`: 最多索引多少行（None表示全部）
    /// 
    /// # 返回
    /// (索引, 是否完成)
    pub fn build_partial(
        mmap: &Mmap,
        has_headers: bool,
        granularity: usize,
        max_rows: Option<usize>,
    ) -> Result<(Self, bool)> {
        let total_bytes = mmap.len();
        
        // 跳过BOM
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3u64
        } else {
            0u64
        };

        // 跳过表头
        let data_start = if has_headers {
            let header_slice = &mmap[start_offset as usize..];
            if let Some(pos) = memchr::memchr(b'\n', header_slice) {
                start_offset + pos as u64 + 1
            } else {
                start_offset
            }
        } else {
            start_offset
        };

        let mut offsets = Vec::new();
        let mut row_numbers = Vec::new();
        let mut current_row = 0;
        let mut line_start = data_start;
        let max_rows = max_rows.unwrap_or(usize::MAX);

        let data_slice = &mmap[data_start as usize..];
        let mut last_newline_pos = None;

        for newline_pos in memchr_iter(b'\n', data_slice) {
            let absolute_pos = data_start as usize + newline_pos;
            let absolute_pos_u64 = absolute_pos as u64;
            
            last_newline_pos = Some(absolute_pos_u64);
            current_row += 1;
            
            if current_row % granularity == 0 {
                offsets.push(line_start);
                row_numbers.push(current_row);
            }
            
            line_start = absolute_pos_u64 + 1;

            // 达到最大行数限制
            if current_row >= max_rows {
                return Ok((Self {
                    offsets,
                    row_numbers,
                    granularity,
                    total_rows: current_row,
                    is_complete: false,
                    indexed_bytes: line_start,
                }, false));
            }
        }

        // 处理最后一行
        if let Some(last_nl) = last_newline_pos {
            if ((last_nl + 1) as usize) < total_bytes {
                current_row += 1;
            }
        } else if (data_start as usize) < total_bytes {
            current_row = 1;
        }

        Ok((Self {
            offsets,
            row_numbers,
            granularity,
            total_rows: current_row,
            is_complete: true,
            indexed_bytes: total_bytes as u64,
        }, true))
    }

    /// 继续构建索引（从上次停止的地方继续）
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `cancel_flag`: 取消标志，设为true时停止构建
    /// - `progress`: 进度报告（已处理字节数）
    pub fn continue_build(
        &mut self,
        mmap: &Mmap,
        cancel_flag: Option<&AtomicBool>,
        progress: Option<&AtomicUsize>,
    ) -> Result<bool> {
        if self.is_complete {
            return Ok(true);
        }

        let total_bytes = mmap.len();
        let start_offset = self.indexed_bytes as usize;

        if start_offset >= total_bytes {
            self.is_complete = true;
            return Ok(true);
        }

        let data_slice = &mmap[start_offset..];
        let mut line_start = self.indexed_bytes;
        let mut current_row = self.total_rows;

        for newline_pos in memchr_iter(b'\n', data_slice) {
            // 检查是否取消
            if let Some(flag) = cancel_flag {
                if flag.load(Ordering::Relaxed) {
                    self.indexed_bytes = line_start;
                    self.total_rows = current_row;
                    return Ok(false);
                }
            }

            let absolute_pos = start_offset + newline_pos;
            let absolute_pos_u64 = absolute_pos as u64;
            
            current_row += 1;
            
            if current_row % self.granularity == 0 {
                self.offsets.push(line_start);
                self.row_numbers.push(current_row);
            }
            
            line_start = absolute_pos_u64 + 1;

            // 更新进度
            if let Some(prog) = progress {
                prog.store(absolute_pos, Ordering::Relaxed);
            }
        }

        // 处理最后一行
        if line_start < total_bytes as u64 {
            current_row += 1;
        }

        self.total_rows = current_row;
        self.indexed_bytes = total_bytes as u64;
        self.is_complete = true;

        if let Some(prog) = progress {
            prog.store(total_bytes, Ordering::Relaxed);
        }

        Ok(true)
    }

    /// 检查索引是否完成
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// 获取已索引的字节数
    pub fn indexed_bytes(&self) -> u64 {
        self.indexed_bytes
    }

    /// 从内存映射文件构建索引
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `granularity`: 索引粒度（每N行记录一次）
    /// 
    /// # 注意
    /// 对于大文件（>100MB），会自动使用并行构建以提高速度
    pub fn build(
        mmap: &Mmap, 
        has_headers: bool, 
        granularity: usize,
    ) -> Result<Self> {
        // 对于大文件（>100MB），使用并行构建
        const PARALLEL_THRESHOLD: usize = 100 * 1024 * 1024; // 100MB
        if mmap.len() > PARALLEL_THRESHOLD {
            Self::build_parallel::<fn(f64, usize, usize)>(mmap, has_headers, granularity, None)
        } else {
            Self::build_with_progress::<fn(f64, usize, usize)>(mmap, has_headers, granularity, None)
        }
    }

    /// 并行构建索引（多线程）
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `granularity`: 索引粒度（每N行记录一次）
    /// - `_progress_callback`: 可选的进度回调函数（当前未实现，保留用于未来扩展）
    /// 
    /// # 性能
    /// 对于大文件（>100MB），使用多线程可以提升 2-4倍速度（取决于CPU核心数）
    pub fn build_parallel<F>(
        mmap: &Mmap,
        has_headers: bool,
        granularity: usize,
        _progress_callback: Option<F>,
    ) -> Result<Self>
    where
        F: FnMut(f64, usize, usize) + Send + Sync,
    {
        let total_bytes = mmap.len();
        
        // 如果有多余的字节，跳过BOM标记
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3u64
        } else {
            0u64
        };

        // 计算数据起始位置（跳过表头）
        let data_start_offset = if has_headers {
            let header_slice = &mmap[start_offset as usize..];
            if let Some(pos) = memchr::memchr(b'\n', header_slice) {
                start_offset + pos as u64 + 1
            } else {
                start_offset
            }
        } else {
            start_offset
        };

        // 确定线程数和块大小
        let num_threads = rayon::current_num_threads();
        let chunk_size = (total_bytes as usize - data_start_offset as usize) / num_threads;
        // 确保块大小至少为1MB，避免过多线程
        let min_chunk_size = 1024 * 1024;
        let effective_chunk_size = chunk_size.max(min_chunk_size);
        
        // 将文件分成多个块
        let mut chunks = Vec::new();
        let mut current_pos = data_start_offset as usize;
        while current_pos < total_bytes {
            let chunk_end = (current_pos + effective_chunk_size).min(total_bytes);
            chunks.push((current_pos, chunk_end));
            current_pos = chunk_end;
        }

        // 并行处理每个块，查找所有换行符位置
        let chunk_results: Vec<_> = chunks
            .into_par_iter()
            .map(|(chunk_start, chunk_end)| {
                // 处理块边界：如果不是第一个块，需要找到第一个完整的行
                let mut actual_start = chunk_start;
                if chunk_start > data_start_offset as usize {
                    // 向前查找换行符（最多向前查找1KB）
                    let search_start = chunk_start.saturating_sub(1024);
                    if let Some(pos) = memchr::memchr(b'\n', &mmap[search_start..chunk_start]) {
                        actual_start = search_start + pos + 1;
                    }
                }

                // 查找该块内的所有换行符位置
                let mut newline_positions = Vec::new();
                let chunk_data = &mmap[actual_start..chunk_end];
                for pos in memchr_iter(b'\n', chunk_data) {
                    newline_positions.push(actual_start + pos);
                }

                (actual_start, newline_positions)
            })
            .collect();

        // 合并所有块的结果，按位置排序
        let mut all_newlines: Vec<usize> = chunk_results
            .into_iter()
            .flat_map(|(_, newlines)| newlines)
            .collect();
        all_newlines.sort_unstable();

        // 计算索引点和行号
        let mut offsets = Vec::new();
        let mut row_numbers = Vec::new();
        let mut current_row = 0;
        let mut line_start = data_start_offset;

        for &nl_pos in &all_newlines {
            current_row += 1;
            let nl_pos_u64 = nl_pos as u64;
            
            // 每N行记录一次索引点
            if current_row % granularity == 0 {
                offsets.push(line_start);
                row_numbers.push(current_row);
            }
            
            // 更新下一行的起始位置
            line_start = nl_pos_u64 + 1;
        }

        // 处理最后一行（如果文件末尾没有换行符，但还有内容）
        let total_rows = if let Some(&last_nl) = all_newlines.last() {
            if (last_nl + 1) < total_bytes {
                all_newlines.len() + 1
            } else {
                all_newlines.len()
            }
        } else {
            // 如果没有找到任何换行符，但文件不为空，说明只有一行
            if (data_start_offset as usize) < total_bytes {
                1
            } else {
                0
            }
        };

        // 注意：进度回调在多线程环境下难以实现，这里暂时忽略
        // 如果需要进度显示，可以使用单线程版本

        Ok(Self {
            offsets,
            row_numbers,
            granularity,
            total_rows,
            is_complete: true,
            indexed_bytes: total_bytes as u64,
        })
    }

    /// 从内存映射文件构建索引（带进度回调）
    /// 
    /// # 参数
    /// - `mmap`: 内存映射的文件
    /// - `has_headers`: 是否有表头
    /// - `granularity`: 索引粒度（每N行记录一次）
    /// - `progress_callback`: 可选的进度回调函数 (进度百分比, 已处理字节数, 总字节数)
    pub fn build_with_progress<F>(
        mmap: &Mmap, 
        has_headers: bool, 
        granularity: usize,
        mut progress_callback: Option<F>,
    ) -> Result<Self>
    where
        F: FnMut(f64, usize, usize),
    {
        let mut offsets = Vec::new();
        let mut row_numbers = Vec::new();
        let mut current_row = 0;
        let current_offset: u64;
        let mut line_start: u64;

        let total_bytes = mmap.len();
        let progress_interval = (total_bytes / 100).max(1024 * 1024); // 每1%或每1MB更新一次进度
        let mut last_progress_update = 0usize;

        // 如果有多余的字节，跳过BOM标记
        let start_offset = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            3u64
        } else {
            0u64
        };

        // 跳过表头（如果存在）- 使用memchr加速
        if has_headers {
            let header_slice = &mmap[start_offset as usize..];
            if let Some(pos) = memchr::memchr(b'\n', header_slice) {
                line_start = start_offset + pos as u64 + 1;
                current_offset = line_start;
            } else {
                // 如果没有找到换行符，整个文件就是一行
                current_offset = start_offset;
                line_start = start_offset;
            }
        } else {
            current_offset = start_offset;
            line_start = start_offset;
        }

        // 扫描文件，记录索引点 - 使用memchr批量查找换行符
        let data_slice = &mmap[current_offset as usize..];
        let mut last_newline_pos = None;
        
        // 批量处理换行符，减少循环开销
        for newline_pos in memchr_iter(b'\n', data_slice) {
            let absolute_pos = current_offset as usize + newline_pos;
            let absolute_pos_u64 = absolute_pos as u64;
            
            // 更新进度（每1MB或1%更新一次）
            if let Some(ref mut callback) = progress_callback {
                if absolute_pos - last_progress_update >= progress_interval {
                    let progress = (absolute_pos as f64 / total_bytes as f64) * 100.0;
                    callback(progress, absolute_pos, total_bytes);
                    last_progress_update = absolute_pos;
                }
            }
            
            last_newline_pos = Some(absolute_pos_u64);
            current_row += 1;
            
            // 每N行记录一次索引点
            if current_row % granularity == 0 {
                offsets.push(line_start);
                row_numbers.push(current_row);
            }
            
            // 更新下一行的起始位置
            line_start = absolute_pos_u64 + 1;
        }
        
        // 最终进度更新
        if let Some(ref mut callback) = progress_callback {
            callback(100.0, total_bytes, total_bytes);
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
            is_complete: true,
            indexed_bytes: total_bytes as u64,
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

