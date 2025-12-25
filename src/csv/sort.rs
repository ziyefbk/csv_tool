//! CSV数据排序模块
//! 
//! 支持按列排序（升序/降序），支持多种数据类型

use crate::csv::{CsvReader, CsvRecord, SearchPattern, SearchOptions};
use crate::error::Result;
use std::cmp::Ordering;

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    /// 升序
    Ascending,
    /// 降序
    Descending,
}

impl SortOrder {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" | "a" => Some(SortOrder::Ascending),
            "desc" | "descending" | "d" => Some(SortOrder::Descending),
            _ => None,
        }
    }

    /// 反转排序方向
    pub fn reverse(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// 数据类型（用于排序）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    /// 字符串（字典序）
    String,
    /// 数字（数值比较）
    Number,
    /// 自动检测
    Auto,
}

impl DataType {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "string" | "str" | "s" | "text" => Some(DataType::String),
            "number" | "num" | "n" | "numeric" => Some(DataType::Number),
            "auto" | "a" => Some(DataType::Auto),
            _ => None,
        }
    }
}

/// 排序键
#[derive(Debug, Clone)]
pub struct SortKey {
    /// 列索引
    pub column: usize,
    /// 排序方向
    pub order: SortOrder,
    /// 数据类型
    pub data_type: DataType,
}

impl SortKey {
    /// 创建新的排序键
    pub fn new(column: usize, order: SortOrder, data_type: DataType) -> Self {
        Self { column, order, data_type }
    }

    /// 创建升序排序键
    pub fn ascending(column: usize) -> Self {
        Self::new(column, SortOrder::Ascending, DataType::Auto)
    }

    /// 创建降序排序键
    pub fn descending(column: usize) -> Self {
        Self::new(column, SortOrder::Descending, DataType::Auto)
    }

    /// 设置数据类型
    pub fn with_data_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }
}

/// 排序选项
#[derive(Debug, Clone)]
pub struct SortOptions {
    /// 排序键列表（支持多列排序）
    pub keys: Vec<SortKey>,
    /// 空值排在最后
    pub nulls_last: bool,
    /// 大小写敏感
    pub case_sensitive: bool,
}

impl Default for SortOptions {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            nulls_last: true,
            case_sensitive: true,
        }
    }
}

impl SortOptions {
    /// 创建新的排序选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加排序键
    pub fn add_key(mut self, key: SortKey) -> Self {
        self.keys.push(key);
        self
    }

    /// 设置空值位置
    pub fn with_nulls_last(mut self, nulls_last: bool) -> Self {
        self.nulls_last = nulls_last;
        self
    }

    /// 设置大小写敏感性
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }
}

/// 排序后的结果
#[derive(Debug, Clone)]
pub struct SortedRecord {
    /// 原始行号
    pub original_row: usize,
    /// 记录数据
    pub record: CsvRecord<'static>,
}

/// 排序器
pub struct Sorter {
    options: SortOptions,
}

impl Sorter {
    /// 创建新的排序器
    pub fn new(options: SortOptions) -> Self {
        Self { options }
    }

    /// 对记录进行排序
    pub fn sort(&self, records: Vec<(usize, CsvRecord<'static>)>) -> Vec<SortedRecord> {
        let mut indexed: Vec<SortedRecord> = records
            .into_iter()
            .map(|(idx, record)| SortedRecord {
                original_row: idx,
                record,
            })
            .collect();

        indexed.sort_by(|a, b| self.compare_records(&a.record, &b.record));

        indexed
    }

    /// 比较两条记录
    fn compare_records(&self, a: &CsvRecord, b: &CsvRecord) -> Ordering {
        for key in &self.options.keys {
            let field_a = a.fields.get(key.column).map(|f| f.as_ref());
            let field_b = b.fields.get(key.column).map(|f| f.as_ref());

            let ordering = self.compare_fields(field_a, field_b, key);
            
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        Ordering::Equal
    }

    /// 比较两个字段值
    fn compare_fields(&self, a: Option<&str>, b: Option<&str>, key: &SortKey) -> Ordering {
        // 处理空值和空字符串 - 这些不受排序方向影响
        match (a, b) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => {
                return if self.options.nulls_last {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
            (Some(_), None) => {
                return if self.options.nulls_last {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
            (Some(a_str), Some(b_str)) => {
                // 检查空字符串 - 也不受排序方向影响
                let a_empty = a_str.is_empty();
                let b_empty = b_str.is_empty();
                
                if a_empty && b_empty {
                    return Ordering::Equal;
                } else if a_empty {
                    return if self.options.nulls_last {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    };
                } else if b_empty {
                    return if self.options.nulls_last {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    };
                }
                
                // 正常值比较 - 受排序方向影响
                let ordering = self.compare_values(a_str, b_str, key);
                match key.order {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            }
        }
    }

    /// 比较两个非空值
    fn compare_values(&self, a: &str, b: &str, key: &SortKey) -> Ordering {
        match key.data_type {
            DataType::String => self.compare_strings(a, b),
            DataType::Number => self.compare_numbers(a, b),
            DataType::Auto => {
                // 尝试作为数字比较
                if let (Ok(num_a), Ok(num_b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal)
                } else {
                    self.compare_strings(a, b)
                }
            }
        }
    }

    /// 字符串比较
    fn compare_strings(&self, a: &str, b: &str) -> Ordering {
        if self.options.case_sensitive {
            a.cmp(b)
        } else {
            a.to_lowercase().cmp(&b.to_lowercase())
        }
    }

    /// 数字比较
    fn compare_numbers(&self, a: &str, b: &str) -> Ordering {
        let num_a = a.parse::<f64>().unwrap_or(f64::NAN);
        let num_b = b.parse::<f64>().unwrap_or(f64::NAN);

        // 处理 NaN
        match (num_a.is_nan(), num_b.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => {
                if self.options.nulls_last {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (false, true) => {
                if self.options.nulls_last {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (false, false) => num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal),
        }
    }
}

/// 从 CsvReader 读取并排序数据
pub fn sort_csv_data(
    reader: &CsvReader,
    options: &SortOptions,
    limit: Option<usize>,
) -> Result<Vec<SortedRecord>> {
    // 读取所有数据
    let pattern = SearchPattern::regex(".*", true)?;
    let search_opts = SearchOptions::new(pattern);
    
    let results = reader.search(&search_opts)?;
    
    let records: Vec<(usize, CsvRecord<'static>)> = results
        .into_iter()
        .map(|r| (r.row_number, r.record))
        .collect();

    // 排序
    let sorter = Sorter::new(options.clone());
    let mut sorted = sorter.sort(records);

    // 限制结果数量
    if let Some(n) = limit {
        sorted.truncate(n);
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order() {
        assert_eq!(SortOrder::from_str("asc"), Some(SortOrder::Ascending));
        assert_eq!(SortOrder::from_str("DESC"), Some(SortOrder::Descending));
        assert_eq!(SortOrder::from_str("invalid"), None);
    }

    #[test]
    fn test_data_type() {
        assert_eq!(DataType::from_str("string"), Some(DataType::String));
        assert_eq!(DataType::from_str("number"), Some(DataType::Number));
        assert_eq!(DataType::from_str("auto"), Some(DataType::Auto));
    }

    #[test]
    fn test_number_comparison() {
        let sorter = Sorter::new(SortOptions::new());
        let key = SortKey::new(0, SortOrder::Ascending, DataType::Number);
        
        assert_eq!(sorter.compare_values("10", "2", &key), Ordering::Greater);
        assert_eq!(sorter.compare_values("2", "10", &key), Ordering::Less);
        assert_eq!(sorter.compare_values("5", "5", &key), Ordering::Equal);
    }

    #[test]
    fn test_string_comparison() {
        let sorter = Sorter::new(SortOptions::new());
        let key = SortKey::new(0, SortOrder::Ascending, DataType::String);
        
        assert_eq!(sorter.compare_values("apple", "banana", &key), Ordering::Less);
        assert_eq!(sorter.compare_values("banana", "apple", &key), Ordering::Greater);
    }

    #[test]
    fn test_case_insensitive() {
        let sorter = Sorter::new(SortOptions::new().with_case_sensitive(false));
        let key = SortKey::new(0, SortOrder::Ascending, DataType::String);
        
        assert_eq!(sorter.compare_values("Apple", "apple", &key), Ordering::Equal);
    }
}

