//! CSV写入模块
//! 
//! 支持CSV数据的修改和保存，包括：
//! - 单元格编辑
//! - 行添加/删除
//! - 列添加/删除
//! - 流式写入（大文件支持）

use crate::csv::{CsvReader, CsvRecord};
use crate::error::{CsvError, Result};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// 单元格修改记录
#[derive(Debug, Clone)]
pub struct CellEdit {
    /// 行号（0-based，不含表头）
    pub row: usize,
    /// 列号（0-based）
    pub col: usize,
    /// 新值
    pub value: String,
}

/// 行数据
#[derive(Debug, Clone)]
pub struct RowData {
    /// 字段值列表
    pub fields: Vec<String>,
}

impl RowData {
    /// 创建新行
    pub fn new(fields: Vec<String>) -> Self {
        Self { fields }
    }

    /// 从字符串数组创建
    pub fn from_strs(fields: &[&str]) -> Self {
        Self {
            fields: fields.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// 获取字段数量
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl From<CsvRecord<'_>> for RowData {
    fn from(record: CsvRecord<'_>) -> Self {
        Self {
            fields: record.fields.into_iter().map(|f| f.into_owned()).collect(),
        }
    }
}

/// 写入选项
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// 分隔符
    pub delimiter: u8,
    /// 行结束符
    pub line_ending: LineEnding,
    /// 是否总是引用字段
    pub always_quote: bool,
    /// 是否写入表头
    pub write_headers: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            delimiter: b',',
            line_ending: LineEnding::default(),
            always_quote: false,
            write_headers: true,
        }
    }
}

impl WriteOptions {
    /// 创建新选项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置分隔符
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// 设置行结束符
    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    /// 设置是否总是引用
    pub fn with_always_quote(mut self, always_quote: bool) -> Self {
        self.always_quote = always_quote;
        self
    }

    /// 设置是否写入表头
    pub fn with_headers(mut self, write_headers: bool) -> Self {
        self.write_headers = write_headers;
        self
    }
}

/// 行结束符类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    /// Unix风格 (\n)
    Lf,
    /// Windows风格 (\r\n)
    CrLf,
    /// 旧Mac风格 (\r)
    Cr,
}

impl Default for LineEnding {
    fn default() -> Self {
        #[cfg(windows)]
        return LineEnding::CrLf;
        #[cfg(not(windows))]
        return LineEnding::Lf;
    }
}

impl LineEnding {
    /// 获取字节表示
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            LineEnding::Lf => b"\n",
            LineEnding::CrLf => b"\r\n",
            LineEnding::Cr => b"\r",
        }
    }
}

/// CSV编辑器
/// 
/// 使用修改追踪模式，只在内存中保存修改，
/// 保存时将原始数据和修改合并写入新文件。
pub struct CsvEditor {
    /// 源文件路径
    source_path: String,
    /// 表头
    headers: Vec<String>,
    /// 原始列数
    original_col_count: usize,
    /// 原始行数（不含表头）
    original_row_count: usize,
    /// 分隔符
    delimiter: u8,
    /// 是否有表头
    has_headers: bool,
    /// 索引粒度
    granularity: usize,
    
    /// 单元格修改记录 (row, col) -> value
    cell_edits: HashMap<(usize, usize), String>,
    /// 新增的行 (插入位置 -> 行数据列表)
    inserted_rows: HashMap<usize, Vec<RowData>>,
    /// 删除的行号集合
    deleted_rows: HashSet<usize>,
    /// 新增的列 (插入位置 -> (列名, 默认值))
    inserted_cols: HashMap<usize, (String, String)>,
    /// 删除的列号集合
    deleted_cols: HashSet<usize>,
    /// 追加的行
    appended_rows: Vec<RowData>,
}

impl CsvEditor {
    /// 打开CSV文件进行编辑
    pub fn open<P: AsRef<Path>>(
        path: P,
        has_headers: bool,
        delimiter: u8,
        granularity: usize,
    ) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // 使用CsvReader读取基本信息
        let reader = CsvReader::open(&path_str, has_headers, delimiter, granularity)?;
        let info = reader.info();
        
        Ok(Self {
            source_path: path_str,
            headers: info.headers.clone(),
            original_col_count: info.total_cols,
            original_row_count: info.total_rows,
            delimiter,
            has_headers,
            granularity,
            cell_edits: HashMap::new(),
            inserted_rows: HashMap::new(),
            deleted_rows: HashSet::new(),
            inserted_cols: HashMap::new(),
            deleted_cols: HashSet::new(),
            appended_rows: Vec::new(),
        })
    }

    /// 获取表头
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// 获取原始行数
    pub fn row_count(&self) -> usize {
        self.original_row_count
    }

    /// 获取原始列数
    pub fn col_count(&self) -> usize {
        self.original_col_count
    }

    /// 获取有效行数（考虑删除和新增）
    pub fn effective_row_count(&self) -> usize {
        let deleted = self.deleted_rows.len();
        let inserted: usize = self.inserted_rows.values().map(|v| v.len()).sum();
        let appended = self.appended_rows.len();
        
        self.original_row_count - deleted + inserted + appended
    }

    /// 检查是否有未保存的修改
    pub fn has_changes(&self) -> bool {
        !self.cell_edits.is_empty()
            || !self.inserted_rows.is_empty()
            || !self.deleted_rows.is_empty()
            || !self.inserted_cols.is_empty()
            || !self.deleted_cols.is_empty()
            || !self.appended_rows.is_empty()
    }

    /// 编辑单元格
    pub fn edit_cell(&mut self, row: usize, col: usize, value: String) -> Result<()> {
        if row >= self.original_row_count && !self.is_appended_row(row) {
            return Err(CsvError::IndexOutOfBounds {
                row,
                total_rows: self.effective_row_count(),
            });
        }
        
        if col >= self.original_col_count && !self.deleted_cols.contains(&col) {
            return Err(CsvError::Format(format!(
                "列 {} 超出范围（总列数: {}）",
                col, self.original_col_count
            )));
        }
        
        if self.deleted_rows.contains(&row) {
            return Err(CsvError::Format(format!("行 {} 已被删除", row)));
        }
        
        self.cell_edits.insert((row, col), value);
        Ok(())
    }

    /// 检查是否是追加的行
    fn is_appended_row(&self, row: usize) -> bool {
        row >= self.original_row_count && row < self.original_row_count + self.appended_rows.len()
    }

    /// 获取单元格值（考虑修改）
    pub fn get_cell(&self, row: usize, col: usize) -> Result<Option<String>> {
        // 检查是否已删除
        if self.deleted_rows.contains(&row) {
            return Ok(None);
        }
        
        // 检查是否有编辑
        if let Some(value) = self.cell_edits.get(&(row, col)) {
            return Ok(Some(value.clone()));
        }
        
        // 检查是否是追加的行
        if row >= self.original_row_count {
            let appended_idx = row - self.original_row_count;
            if appended_idx < self.appended_rows.len() {
                return Ok(self.appended_rows[appended_idx].fields.get(col).cloned());
            }
            return Ok(None);
        }
        
        // 从原始文件读取
        let mut reader = CsvReader::open(
            &self.source_path,
            self.has_headers,
            self.delimiter,
            self.granularity,
        )?;
        
        let page = reader.read_page(row, 1)?;
        if let Some(record) = page.first() {
            Ok(record.fields.get(col).map(|f| f.to_string()))
        } else {
            Ok(None)
        }
    }

    /// 删除行
    pub fn delete_row(&mut self, row: usize) -> Result<()> {
        if row >= self.original_row_count {
            // 删除追加的行
            let appended_idx = row - self.original_row_count;
            if appended_idx < self.appended_rows.len() {
                self.appended_rows.remove(appended_idx);
                return Ok(());
            }
            return Err(CsvError::IndexOutOfBounds {
                row,
                total_rows: self.effective_row_count(),
            });
        }
        
        self.deleted_rows.insert(row);
        
        // 清除该行的所有编辑
        self.cell_edits.retain(|&(r, _), _| r != row);
        
        Ok(())
    }

    /// 恢复删除的行
    pub fn undelete_row(&mut self, row: usize) -> Result<()> {
        if !self.deleted_rows.remove(&row) {
            return Err(CsvError::Format(format!("行 {} 未被删除", row)));
        }
        Ok(())
    }

    /// 追加新行
    pub fn append_row(&mut self, row: RowData) -> Result<()> {
        // 确保列数匹配
        let expected_cols = self.effective_col_count();
        if row.len() != expected_cols {
            return Err(CsvError::Format(format!(
                "新行列数 {} 与表格列数 {} 不匹配",
                row.len(), expected_cols
            )));
        }
        
        self.appended_rows.push(row);
        Ok(())
    }

    /// 在指定位置插入行
    pub fn insert_row(&mut self, position: usize, row: RowData) -> Result<()> {
        if position > self.original_row_count {
            return Err(CsvError::IndexOutOfBounds {
                row: position,
                total_rows: self.original_row_count,
            });
        }
        
        let expected_cols = self.effective_col_count();
        if row.len() != expected_cols {
            return Err(CsvError::Format(format!(
                "新行列数 {} 与表格列数 {} 不匹配",
                row.len(), expected_cols
            )));
        }
        
        self.inserted_rows
            .entry(position)
            .or_insert_with(Vec::new)
            .push(row);
        
        Ok(())
    }

    /// 删除列
    pub fn delete_col(&mut self, col: usize) -> Result<()> {
        if col >= self.original_col_count {
            return Err(CsvError::Format(format!(
                "列 {} 超出范围（总列数: {}）",
                col, self.original_col_count
            )));
        }
        
        self.deleted_cols.insert(col);
        
        // 清除该列的所有编辑
        self.cell_edits.retain(|&(_, c), _| c != col);
        
        // 更新表头
        if col < self.headers.len() {
            // 标记为删除，实际删除在保存时处理
        }
        
        Ok(())
    }

    /// 获取有效列数
    pub fn effective_col_count(&self) -> usize {
        let deleted = self.deleted_cols.len();
        let inserted = self.inserted_cols.len();
        
        self.original_col_count - deleted + inserted
    }

    /// 修改表头
    pub fn set_header(&mut self, col: usize, name: String) -> Result<()> {
        if col >= self.headers.len() {
            return Err(CsvError::Format(format!(
                "列 {} 超出范围（总列数: {}）",
                col, self.headers.len()
            )));
        }
        
        self.headers[col] = name;
        Ok(())
    }

    /// 清除所有修改
    pub fn discard_changes(&mut self) {
        self.cell_edits.clear();
        self.inserted_rows.clear();
        self.deleted_rows.clear();
        self.inserted_cols.clear();
        self.deleted_cols.clear();
        self.appended_rows.clear();
    }

    /// 获取修改统计
    pub fn change_stats(&self) -> ChangeStats {
        ChangeStats {
            cells_edited: self.cell_edits.len(),
            rows_deleted: self.deleted_rows.len(),
            rows_inserted: self.inserted_rows.values().map(|v| v.len()).sum(),
            rows_appended: self.appended_rows.len(),
            cols_deleted: self.deleted_cols.len(),
            cols_inserted: self.inserted_cols.len(),
        }
    }

    /// 保存到文件
    pub fn save<P: AsRef<Path>>(&self, output_path: P, options: &WriteOptions) -> Result<SaveStats> {
        let file = File::create(output_path.as_ref())?;
        let mut writer = BufWriter::new(file);
        
        let mut rows_written = 0;
        let mut bytes_written = 0;
        
        // 写入表头
        if options.write_headers && !self.headers.is_empty() {
            let effective_headers: Vec<&str> = self.headers
                .iter()
                .enumerate()
                .filter(|(i, _)| !self.deleted_cols.contains(i))
                .map(|(_, h)| h.as_str())
                .collect();
            
            let line = self.format_row(&effective_headers, options);
            writer.write_all(line.as_bytes())?;
            writer.write_all(options.line_ending.as_bytes())?;
            bytes_written += line.len() + options.line_ending.as_bytes().len();
        }
        
        // 打开源文件读取器
        let mut reader = CsvReader::open(
            &self.source_path,
            self.has_headers,
            self.delimiter,
            self.granularity,
        )?;
        
        // 逐行处理
        let mut current_row = 0;
        while current_row < self.original_row_count {
            // 检查是否有插入的行
            if let Some(inserted) = self.inserted_rows.get(&current_row) {
                for row in inserted {
                    let fields: Vec<&str> = row.fields
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !self.deleted_cols.contains(i))
                        .map(|(_, f)| f.as_str())
                        .collect();
                    
                    let line = self.format_row(&fields, options);
                    writer.write_all(line.as_bytes())?;
                    writer.write_all(options.line_ending.as_bytes())?;
                    bytes_written += line.len() + options.line_ending.as_bytes().len();
                    rows_written += 1;
                }
            }
            
            // 跳过删除的行
            if self.deleted_rows.contains(&current_row) {
                current_row += 1;
                continue;
            }
            
            // 读取并处理当前行
            let page = reader.read_page(current_row, 1)?;
            if let Some(record) = page.first() {
                let fields: Vec<Cow<str>> = record.fields
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !self.deleted_cols.contains(i))
                    .map(|(i, f)| {
                        // 检查是否有编辑
                        if let Some(edited) = self.cell_edits.get(&(current_row, i)) {
                            Cow::Owned(edited.clone())
                        } else {
                            Cow::Borrowed(f.as_ref())
                        }
                    })
                    .collect();
                
                let field_strs: Vec<&str> = fields.iter().map(|f| f.as_ref()).collect();
                let line = self.format_row(&field_strs, options);
                writer.write_all(line.as_bytes())?;
                writer.write_all(options.line_ending.as_bytes())?;
                bytes_written += line.len() + options.line_ending.as_bytes().len();
                rows_written += 1;
            }
            
            current_row += 1;
        }
        
        // 写入追加的行
        for row in &self.appended_rows {
            let fields: Vec<&str> = row.fields
                .iter()
                .enumerate()
                .filter(|(i, _)| !self.deleted_cols.contains(i))
                .map(|(_, f)| f.as_str())
                .collect();
            
            let line = self.format_row(&fields, options);
            writer.write_all(line.as_bytes())?;
            writer.write_all(options.line_ending.as_bytes())?;
            bytes_written += line.len() + options.line_ending.as_bytes().len();
            rows_written += 1;
        }
        
        writer.flush()?;
        
        Ok(SaveStats {
            rows_written,
            bytes_written,
            file_path: output_path.as_ref().to_string_lossy().to_string(),
        })
    }

    /// 保存到原文件（覆盖）
    pub fn save_in_place(&self, options: &WriteOptions) -> Result<SaveStats> {
        // 先保存到临时文件
        let temp_path = format!("{}.tmp", self.source_path);
        let stats = self.save(&temp_path, options)?;
        
        // 重命名临时文件覆盖原文件
        std::fs::rename(&temp_path, &self.source_path)?;
        
        Ok(SaveStats {
            file_path: self.source_path.clone(),
            ..stats
        })
    }

    /// 格式化一行数据
    fn format_row(&self, fields: &[&str], options: &WriteOptions) -> String {
        let delimiter = options.delimiter as char;
        
        fields
            .iter()
            .map(|field| self.escape_field(field, options))
            .collect::<Vec<_>>()
            .join(&delimiter.to_string())
    }

    /// 转义字段值
    fn escape_field(&self, field: &str, options: &WriteOptions) -> String {
        let delimiter = options.delimiter as char;
        let needs_quote = options.always_quote
            || field.contains(delimiter)
            || field.contains('"')
            || field.contains('\n')
            || field.contains('\r');
        
        if needs_quote {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

/// 修改统计
#[derive(Debug, Clone)]
pub struct ChangeStats {
    /// 编辑的单元格数
    pub cells_edited: usize,
    /// 删除的行数
    pub rows_deleted: usize,
    /// 插入的行数
    pub rows_inserted: usize,
    /// 追加的行数
    pub rows_appended: usize,
    /// 删除的列数
    pub cols_deleted: usize,
    /// 插入的列数
    pub cols_inserted: usize,
}

impl ChangeStats {
    /// 检查是否有修改
    pub fn has_changes(&self) -> bool {
        self.cells_edited > 0
            || self.rows_deleted > 0
            || self.rows_inserted > 0
            || self.rows_appended > 0
            || self.cols_deleted > 0
            || self.cols_inserted > 0
    }
}

/// 保存统计
#[derive(Debug, Clone)]
pub struct SaveStats {
    /// 写入的行数
    pub rows_written: usize,
    /// 写入的字节数
    pub bytes_written: usize,
    /// 输出文件路径
    pub file_path: String,
}

/// 简单的CSV创建器（从头创建新文件）
pub struct CsvCreator {
    /// 表头
    headers: Vec<String>,
    /// 数据行
    rows: Vec<RowData>,
    /// 写入选项
    options: WriteOptions,
}

impl CsvCreator {
    /// 创建新的CSV创建器
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            options: WriteOptions::default(),
        }
    }

    /// 设置写入选项
    pub fn with_options(mut self, options: WriteOptions) -> Self {
        self.options = options;
        self
    }

    /// 添加一行数据
    pub fn add_row(&mut self, row: RowData) -> Result<()> {
        if row.len() != self.headers.len() {
            return Err(CsvError::Format(format!(
                "行列数 {} 与表头列数 {} 不匹配",
                row.len(), self.headers.len()
            )));
        }
        self.rows.push(row);
        Ok(())
    }

    /// 批量添加行
    pub fn add_rows(&mut self, rows: Vec<RowData>) -> Result<()> {
        for row in rows {
            self.add_row(row)?;
        }
        Ok(())
    }

    /// 保存到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<SaveStats> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);
        
        let mut bytes_written = 0;
        let delimiter = self.options.delimiter as char;
        
        // 写入表头
        if self.options.write_headers && !self.headers.is_empty() {
            let line = self.headers
                .iter()
                .map(|h| escape_csv_field(h, &self.options))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string());
            
            writer.write_all(line.as_bytes())?;
            writer.write_all(self.options.line_ending.as_bytes())?;
            bytes_written += line.len() + self.options.line_ending.as_bytes().len();
        }
        
        // 写入数据行
        for row in &self.rows {
            let line = row.fields
                .iter()
                .map(|f| escape_csv_field(f, &self.options))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string());
            
            writer.write_all(line.as_bytes())?;
            writer.write_all(self.options.line_ending.as_bytes())?;
            bytes_written += line.len() + self.options.line_ending.as_bytes().len();
        }
        
        writer.flush()?;
        
        Ok(SaveStats {
            rows_written: self.rows.len(),
            bytes_written,
            file_path: path.as_ref().to_string_lossy().to_string(),
        })
    }
}

/// 转义CSV字段
fn escape_csv_field(field: &str, options: &WriteOptions) -> String {
    let delimiter = options.delimiter as char;
    let needs_quote = options.always_quote
        || field.contains(delimiter)
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r');
    
    if needs_quote {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_data() {
        let row = RowData::new(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(row.len(), 2);
        assert!(!row.is_empty());
    }

    #[test]
    fn test_row_data_from_strs() {
        let row = RowData::from_strs(&["hello", "world"]);
        assert_eq!(row.fields, vec!["hello", "world"]);
    }

    #[test]
    fn test_escape_csv_field() {
        let options = WriteOptions::default();
        
        assert_eq!(escape_csv_field("simple", &options), "simple");
        assert_eq!(escape_csv_field("with,comma", &options), "\"with,comma\"");
        assert_eq!(escape_csv_field("with\"quote", &options), "\"with\"\"quote\"");
        assert_eq!(escape_csv_field("with\nnewline", &options), "\"with\nnewline\"");
    }

    #[test]
    fn test_write_options() {
        let options = WriteOptions::new()
            .with_delimiter(b'\t')
            .with_always_quote(true)
            .with_headers(false);
        
        assert_eq!(options.delimiter, b'\t');
        assert!(options.always_quote);
        assert!(!options.write_headers);
    }

    #[test]
    fn test_change_stats() {
        let stats = ChangeStats {
            cells_edited: 5,
            rows_deleted: 2,
            rows_inserted: 3,
            rows_appended: 1,
            cols_deleted: 0,
            cols_inserted: 0,
        };
        
        assert!(stats.has_changes());
    }
}

