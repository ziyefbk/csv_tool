//! CSV数据导出模块
//! 
//! 支持将CSV数据导出为多种格式

use crate::csv::{CsvReader, CsvRecord, SearchOptions};
use crate::error::{CsvError, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    /// 标准JSON数组格式
    Json,
    /// JSON Lines格式（每行一个JSON对象）
    JsonLines,
    /// CSV格式（可自定义分隔符）
    Csv,
    /// 制表符分隔值
    Tsv,
}

impl ExportFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "json" => Some(ExportFormat::Json),
            "jsonl" | "ndjson" => Some(ExportFormat::JsonLines),
            "csv" => Some(ExportFormat::Csv),
            "tsv" => Some(ExportFormat::Tsv),
            _ => None,
        }
    }

    /// 获取格式的文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::JsonLines => "jsonl",
            ExportFormat::Csv => "csv",
            ExportFormat::Tsv => "tsv",
        }
    }

    /// 获取格式名称
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON",
            ExportFormat::JsonLines => "JSON Lines",
            ExportFormat::Csv => "CSV",
            ExportFormat::Tsv => "TSV",
        }
    }
}

/// 导出选项
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// 导出格式
    pub format: ExportFormat,
    /// 要导出的列索引（None表示所有列）
    pub columns: Option<Vec<usize>>,
    /// 行范围 (起始行, 结束行)，从0开始
    pub row_range: Option<(usize, usize)>,
    /// 搜索筛选条件
    pub search_filter: Option<SearchOptions>,
    /// JSON美化输出
    pub pretty: bool,
    /// CSV分隔符（仅CSV格式有效）
    pub delimiter: u8,
    /// 是否包含表头
    pub include_headers: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            columns: None,
            row_range: None,
            search_filter: None,
            pretty: false,
            delimiter: b',',
            include_headers: true,
        }
    }
}

impl ExportOptions {
    /// 创建新的导出选项
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// 设置要导出的列
    pub fn with_columns(mut self, columns: Vec<usize>) -> Self {
        self.columns = Some(columns);
        self
    }

    /// 设置行范围
    pub fn with_row_range(mut self, start: usize, end: usize) -> Self {
        self.row_range = Some((start, end));
        self
    }

    /// 设置搜索筛选
    pub fn with_search_filter(mut self, filter: SearchOptions) -> Self {
        self.search_filter = Some(filter);
        self
    }

    /// 设置JSON美化输出
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// 设置CSV分隔符
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// 设置是否包含表头
    pub fn with_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }
}

/// 导出统计信息
#[derive(Debug, Clone)]
pub struct ExportStats {
    /// 导出的行数
    pub rows_exported: usize,
    /// 导出的列数
    pub cols_exported: usize,
    /// 输出文件大小（字节）
    pub file_size: u64,
}

/// 导出器
pub struct Exporter<'a> {
    reader: &'a CsvReader,
    options: ExportOptions,
}

impl<'a> Exporter<'a> {
    /// 创建新的导出器
    pub fn new(reader: &'a CsvReader, options: ExportOptions) -> Self {
        Self { reader, options }
    }

    /// 导出到文件
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<ExportStats> {
        let path = path.as_ref();
        let file = File::create(path)
            .map_err(|e| CsvError::Io(e))?;
        let mut writer = BufWriter::new(file);

        let stats = match self.options.format {
            ExportFormat::Json => self.export_json(&mut writer)?,
            ExportFormat::JsonLines => self.export_jsonl(&mut writer)?,
            ExportFormat::Csv | ExportFormat::Tsv => self.export_csv(&mut writer)?,
        };

        writer.flush().map_err(|e| CsvError::Io(e))?;

        // 获取文件大小
        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ExportStats {
            rows_exported: stats.0,
            cols_exported: stats.1,
            file_size,
        })
    }

    /// 导出为JSON格式
    fn export_json<W: Write>(&self, writer: &mut W) -> Result<(usize, usize)> {
        let headers = self.get_export_headers();
        let records = self.get_export_records()?;
        
        let cols = headers.len();
        let rows = records.len();

        if self.options.pretty {
            writeln!(writer, "[").map_err(|e| CsvError::Io(e))?;
        } else {
            write!(writer, "[").map_err(|e| CsvError::Io(e))?;
        }

        for (i, record) in records.iter().enumerate() {
            let json_obj = self.record_to_json(&headers, record);
            
            if self.options.pretty {
                if i > 0 {
                    writeln!(writer, ",").map_err(|e| CsvError::Io(e))?;
                }
                write!(writer, "  {}", json_obj).map_err(|e| CsvError::Io(e))?;
            } else {
                if i > 0 {
                    write!(writer, ",").map_err(|e| CsvError::Io(e))?;
                }
                write!(writer, "{}", json_obj).map_err(|e| CsvError::Io(e))?;
            }
        }

        if self.options.pretty {
            writeln!(writer).map_err(|e| CsvError::Io(e))?;
            writeln!(writer, "]").map_err(|e| CsvError::Io(e))?;
        } else {
            writeln!(writer, "]").map_err(|e| CsvError::Io(e))?;
        }

        Ok((rows, cols))
    }

    /// 导出为JSON Lines格式
    fn export_jsonl<W: Write>(&self, writer: &mut W) -> Result<(usize, usize)> {
        let headers = self.get_export_headers();
        let records = self.get_export_records()?;
        
        let cols = headers.len();
        let rows = records.len();

        for record in &records {
            let json_obj = self.record_to_json(&headers, record);
            writeln!(writer, "{}", json_obj).map_err(|e| CsvError::Io(e))?;
        }

        Ok((rows, cols))
    }

    /// 导出为CSV/TSV格式
    fn export_csv<W: Write>(&self, writer: &mut W) -> Result<(usize, usize)> {
        let headers = self.get_export_headers();
        let records = self.get_export_records()?;
        
        let delimiter = if self.options.format == ExportFormat::Tsv {
            b'\t'
        } else {
            self.options.delimiter
        };
        let delimiter_char = delimiter as char;

        let cols = headers.len();
        let mut rows = 0;

        // 写入表头
        if self.options.include_headers && !headers.is_empty() {
            let header_line: Vec<String> = headers.iter()
                .map(|h| escape_csv_field(h, delimiter))
                .collect();
            writeln!(writer, "{}", header_line.join(&delimiter_char.to_string()))
                .map_err(|e| CsvError::Io(e))?;
        }

        // 写入数据行
        for record in &records {
            let fields = self.get_record_fields(record);
            let line: Vec<String> = fields.iter()
                .map(|f| escape_csv_field(f, delimiter))
                .collect();
            writeln!(writer, "{}", line.join(&delimiter_char.to_string()))
                .map_err(|e| CsvError::Io(e))?;
            rows += 1;
        }

        Ok((rows, cols))
    }

    /// 获取要导出的表头
    fn get_export_headers(&self) -> Vec<String> {
        let all_headers = self.reader.headers();
        
        match &self.options.columns {
            Some(cols) => cols.iter()
                .filter_map(|&i| all_headers.get(i).cloned())
                .collect(),
            None => all_headers.to_vec(),
        }
    }

    /// 获取要导出的记录
    fn get_export_records(&self) -> Result<Vec<CsvRecord<'static>>> {
        // 如果有搜索筛选，使用搜索结果
        if let Some(ref search_opts) = self.options.search_filter {
            let results = self.reader.search(search_opts)?;
            let records: Vec<CsvRecord<'static>> = results.into_iter()
                .map(|r| r.record)
                .collect();
            return self.apply_row_range(records);
        }

        // 否则读取所有行（或指定范围）
        let info = self.reader.info();
        let (start, end) = self.options.row_range
            .unwrap_or((0, info.total_rows));
        
        let end = end.min(info.total_rows);
        
        // 直接扫描文件获取记录
        self.scan_records(start, end)
    }

    /// 扫描指定范围的记录
    fn scan_records(&self, start: usize, end: usize) -> Result<Vec<CsvRecord<'static>>> {
        let info = self.reader.info();
        let end = end.min(info.total_rows);
        
        if start >= end {
            return Ok(Vec::new());
        }

        // 使用搜索功能获取所有记录（匹配所有行的正则表达式）
        let all_pattern = crate::csv::SearchPattern::regex(".*", true)?;
        let all_opts = SearchOptions::new(all_pattern)
            .with_max_results(end);
        
        let results = self.reader.search(&all_opts)?;
        
        let records: Vec<CsvRecord<'static>> = results.into_iter()
            .skip(start)
            .take(end - start)
            .map(|r| r.record)
            .collect();
        
        Ok(records)
    }

    /// 应用行范围筛选
    fn apply_row_range(&self, records: Vec<CsvRecord<'static>>) -> Result<Vec<CsvRecord<'static>>> {
        match self.options.row_range {
            Some((start, end)) => {
                let end = end.min(records.len());
                Ok(records.into_iter()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .collect())
            }
            None => Ok(records),
        }
    }

    /// 将记录转换为JSON对象字符串
    fn record_to_json(&self, headers: &[String], record: &CsvRecord) -> String {
        let fields = self.get_record_fields(record);
        
        let pairs: Vec<String> = headers.iter()
            .zip(fields.iter())
            .map(|(h, v)| format!("\"{}\":{}", escape_json_string(h), json_value(v)))
            .collect();
        
        format!("{{{}}}", pairs.join(","))
    }

    /// 获取记录的字段（根据列筛选）
    fn get_record_fields(&self, record: &CsvRecord) -> Vec<String> {
        match &self.options.columns {
            Some(cols) => cols.iter()
                .filter_map(|&i| record.fields.get(i).map(|f| f.to_string()))
                .collect(),
            None => record.fields.iter()
                .map(|f| f.to_string())
                .collect(),
        }
    }
}

/// 转义JSON字符串
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// 将值转换为JSON格式
fn json_value(s: &str) -> String {
    // 尝试解析为数字
    if let Ok(_) = s.parse::<i64>() {
        return s.to_string();
    }
    if let Ok(_) = s.parse::<f64>() {
        return s.to_string();
    }
    // 检查布尔值
    match s.to_lowercase().as_str() {
        "true" => return "true".to_string(),
        "false" => return "false".to_string(),
        "null" | "" => return "null".to_string(),
        _ => {}
    }
    // 默认作为字符串
    format!("\"{}\"", escape_json_string(s))
}

/// 转义CSV字段
fn escape_csv_field(s: &str, delimiter: u8) -> String {
    let delimiter_char = delimiter as char;
    let needs_quote = s.contains(delimiter_char) 
        || s.contains('"') 
        || s.contains('\n') 
        || s.contains('\r');
    
    if needs_quote {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("he\"llo"), "he\\\"llo");
        assert_eq!(escape_json_string("he\\llo"), "he\\\\llo");
        assert_eq!(escape_json_string("he\nllo"), "he\\nllo");
    }

    #[test]
    fn test_json_value() {
        assert_eq!(json_value("123"), "123");
        assert_eq!(json_value("12.34"), "12.34");
        assert_eq!(json_value("true"), "true");
        assert_eq!(json_value("false"), "false");
        assert_eq!(json_value("hello"), "\"hello\"");
        assert_eq!(json_value(""), "null");
    }

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("hello", b','), "hello");
        assert_eq!(escape_csv_field("he,llo", b','), "\"he,llo\"");
        assert_eq!(escape_csv_field("he\"llo", b','), "\"he\"\"llo\"");
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(
            ExportFormat::from_extension(Path::new("test.json")),
            Some(ExportFormat::Json)
        );
        assert_eq!(
            ExportFormat::from_extension(Path::new("test.jsonl")),
            Some(ExportFormat::JsonLines)
        );
        assert_eq!(
            ExportFormat::from_extension(Path::new("test.tsv")),
            Some(ExportFormat::Tsv)
        );
    }
}

