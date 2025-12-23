//! CSV工具实用函数

use crate::error::Result;
use std::path::Path;

/// 格式化文件大小
/// 
/// # 参数
/// - `bytes`: 字节数
/// 
/// # 返回
/// 格式化后的字符串（如 "1.23 GB"）
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 检测CSV文件的分隔符
/// 
/// # 参数
/// - `path`: CSV文件路径
/// 
/// # 返回
/// 检测到的分隔符（逗号、分号、制表符等）
pub fn detect_delimiter<P: AsRef<Path>>(path: P) -> Result<u8> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    
    // 统计各种分隔符的出现次数
    let mut comma_count = first_line.matches(',').count();
    let mut semicolon_count = first_line.matches(';').count();
    let mut tab_count = first_line.matches('\t').count();
    let mut pipe_count = first_line.matches('|').count();
    
    // 读取更多行以获得更准确的统计
    for _ in 0..10 {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        comma_count += line.matches(',').count();
        semicolon_count += line.matches(';').count();
        tab_count += line.matches('\t').count();
        pipe_count += line.matches('|').count();
    }
    
    // 返回出现次数最多的分隔符
    let max_count = comma_count.max(semicolon_count).max(tab_count).max(pipe_count);
    
    if max_count == comma_count && comma_count > 0 {
        Ok(b',')
    } else if max_count == semicolon_count && semicolon_count > 0 {
        Ok(b';')
    } else if max_count == tab_count && tab_count > 0 {
        Ok(b'\t')
    } else if max_count == pipe_count && pipe_count > 0 {
        Ok(b'|')
    } else {
        // 默认返回逗号
        Ok(b',')
    }
}

/// 检测CSV文件是否有表头
/// 
/// # 参数
/// - `path`: CSV文件路径
/// 
/// # 返回
/// 如果有表头返回true，否则返回false
pub fn detect_has_headers<P: AsRef<Path>>(path: P) -> Result<bool> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    
    let mut second_line = String::new();
    reader.read_line(&mut second_line)?;
    
    // 简单的启发式方法：
    // 如果第一行看起来像表头（包含字母，第二行包含数字），则可能有表头
    let first_has_letters = first_line.chars().any(|c| c.is_alphabetic());
    let second_has_numbers = second_line.chars().any(|c| c.is_numeric());
    
    Ok(first_has_letters && second_has_numbers)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}

