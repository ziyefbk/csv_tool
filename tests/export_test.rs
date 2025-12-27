//! 导出功能集成测试

use csv_tool::csv::{CsvReader, ExportFormat, ExportOptions, Exporter};
use csv_tool::error::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn create_test_csv(path: &PathBuf) -> Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, "id,name,age,city")?;
    writeln!(file, "1,Alice,25,Beijing")?;
    writeln!(file, "2,Bob,30,Shanghai")?;
    writeln!(file, "3,Charlie,28,Guangzhou")?;
    
    Ok(())
}

#[test]
fn test_export_json() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_export_json.csv");
    let output_file = std::env::temp_dir().join("test_export_output.json");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let options = ExportOptions::new(ExportFormat::Json);
    let exporter = Exporter::new(&reader, options);
    
    let stats = exporter.export_to_file(&output_file)?;
    
    assert_eq!(stats.rows_exported, 3);
    assert_eq!(stats.cols_exported, 4);
    assert!(stats.file_size > 0);
    
    // 验证文件内容
    let content = fs::read_to_string(&output_file)?;
    assert!(content.starts_with("["));
    assert!(content.contains("\"name\":\"Alice\""));
    assert!(content.contains("\"city\":\"Beijing\""));
    
    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_file(&output_file).ok();
    let idx = csv_tool::csv::RowIndex::index_file_path(&test_file);
    fs::remove_file(&idx).ok();
    
    Ok(())
}

#[test]
fn test_export_jsonl() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_export_jsonl.csv");
    let output_file = std::env::temp_dir().join("test_export_output.jsonl");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let options = ExportOptions::new(ExportFormat::JsonLines);
    let exporter = Exporter::new(&reader, options);
    
    let stats = exporter.export_to_file(&output_file)?;
    
    assert_eq!(stats.rows_exported, 3);
    
    // 验证JSONL格式（每行一个JSON对象）
    let content = fs::read_to_string(&output_file)?;
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("{"));
    assert!(lines[0].ends_with("}"));
    
    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_file(&output_file).ok();
    let idx = csv_tool::csv::RowIndex::index_file_path(&test_file);
    fs::remove_file(&idx).ok();
    
    Ok(())
}

#[test]
fn test_export_tsv() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_export_tsv.csv");
    let output_file = std::env::temp_dir().join("test_export_output.tsv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let options = ExportOptions::new(ExportFormat::Tsv);
    let exporter = Exporter::new(&reader, options);
    
    let stats = exporter.export_to_file(&output_file)?;
    
    assert_eq!(stats.rows_exported, 3);
    
    // 验证TSV格式（制表符分隔）
    let content = fs::read_to_string(&output_file)?;
    assert!(content.contains("\t"));
    assert!(content.contains("id\tname\tage\tcity"));
    
    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_file(&output_file).ok();
    let idx = csv_tool::csv::RowIndex::index_file_path(&test_file);
    fs::remove_file(&idx).ok();
    
    Ok(())
}

#[test]
fn test_export_with_columns() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_export_cols.csv");
    let output_file = std::env::temp_dir().join("test_export_cols.json");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let options = ExportOptions::new(ExportFormat::Json)
        .with_columns(vec![0, 1]); // 只导出 id 和 name 列
    let exporter = Exporter::new(&reader, options);
    
    let stats = exporter.export_to_file(&output_file)?;
    
    assert_eq!(stats.cols_exported, 2);
    
    // 验证只包含选择的列
    let content = fs::read_to_string(&output_file)?;
    assert!(content.contains("\"id\":"));
    assert!(content.contains("\"name\":"));
    assert!(!content.contains("\"city\":")); // 不应包含city
    
    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_file(&output_file).ok();
    let idx = csv_tool::csv::RowIndex::index_file_path(&test_file);
    fs::remove_file(&idx).ok();
    
    Ok(())
}

#[test]
fn test_export_with_row_range() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_export_range.csv");
    let output_file = std::env::temp_dir().join("test_export_range.json");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let options = ExportOptions::new(ExportFormat::Json)
        .with_row_range(0, 2); // 只导出前2行
    let exporter = Exporter::new(&reader, options);
    
    let stats = exporter.export_to_file(&output_file)?;
    
    assert_eq!(stats.rows_exported, 2);
    
    // 验证只包含前2行
    let content = fs::read_to_string(&output_file)?;
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    assert!(!content.contains("Charlie")); // 不应包含第3行
    
    // 清理
    fs::remove_file(&test_file).ok();
    fs::remove_file(&output_file).ok();
    let idx = csv_tool::csv::RowIndex::index_file_path(&test_file);
    fs::remove_file(&idx).ok();
    
    Ok(())
}

#[test]
fn test_export_format_detection() {
    assert_eq!(
        ExportFormat::from_extension(std::path::Path::new("test.json")),
        Some(ExportFormat::Json)
    );
    assert_eq!(
        ExportFormat::from_extension(std::path::Path::new("test.jsonl")),
        Some(ExportFormat::JsonLines)
    );
    assert_eq!(
        ExportFormat::from_extension(std::path::Path::new("test.tsv")),
        Some(ExportFormat::Tsv)
    );
    assert_eq!(
        ExportFormat::from_extension(std::path::Path::new("test.csv")),
        Some(ExportFormat::Csv)
    );
}


