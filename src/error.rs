use thiserror::Error;

/// CSV工具的错误类型
#[derive(Error, Debug)]
pub enum CsvError {
    /// IO错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    /// CSV解析错误
    #[error("CSV解析错误: {0}")]
    Parse(#[from] csv::Error),

    /// 索引错误：行号超出范围
    #[error("索引错误: 行 {row} 超出范围（总行数: {total_rows}）")]
    IndexOutOfBounds { row: usize, total_rows: usize },

    /// 内存映射错误
    #[error("内存映射失败: {0}")]
    Mmap(String),

    /// 文件格式错误
    #[error("文件格式错误: {0}")]
    Format(String),

    /// 索引文件错误
    #[error("索引文件错误: {0}")]
    IndexFile(String),
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, CsvError>;

