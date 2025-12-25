//! CSV写入功能集成测试

use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use csv_tool::csv::{CsvEditor, CsvCreator, RowData, WriteOptions};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 创建临时测试文件
fn create_test_csv(content: &str) -> String {
    let counter = TEST_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
    let path = format!("target/test_writer_{}_{}.csv", std::process::id(), counter);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// 创建输出路径
fn output_path() -> String {
    let counter = TEST_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
    format!("target/test_writer_out_{}_{}.csv", std::process::id(), counter)
}

/// 清理测试文件
fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_edit_cell() {
    let content = "name,age\nAlice,25\nBob,30\n";
    let path = create_test_csv(content);
    let out = output_path();
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    editor.edit_cell(0, 0, "Alice Updated".to_string()).unwrap();
    
    let options = WriteOptions::default();
    let stats = editor.save(&out, &options).unwrap();
    
    assert_eq!(stats.rows_written, 2);
    
    // 读取并验证
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("Alice Updated"));
    
    cleanup(&path);
    cleanup(&out);
}

#[test]
fn test_delete_row() {
    let content = "name,age\nAlice,25\nBob,30\nCharlie,35\n";
    let path = create_test_csv(content);
    let out = output_path();
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    editor.delete_row(1).unwrap(); // 删除Bob
    
    let options = WriteOptions::default();
    let stats = editor.save(&out, &options).unwrap();
    
    assert_eq!(stats.rows_written, 2);
    
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("Alice"));
    assert!(!content.contains("Bob"));
    assert!(content.contains("Charlie"));
    
    cleanup(&path);
    cleanup(&out);
}

#[test]
fn test_append_row() {
    let content = "name,age\nAlice,25\n";
    let path = create_test_csv(content);
    let out = output_path();
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    let new_row = RowData::new(vec!["Bob".to_string(), "30".to_string()]);
    editor.append_row(new_row).unwrap();
    
    let options = WriteOptions::default();
    let stats = editor.save(&out, &options).unwrap();
    
    assert_eq!(stats.rows_written, 2);
    
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("Alice"));
    assert!(content.contains("Bob"));
    
    cleanup(&path);
    cleanup(&out);
}

#[test]
fn test_delete_col() {
    let content = "name,age,city\nAlice,25,Beijing\n";
    let path = create_test_csv(content);
    let out = output_path();
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    editor.delete_col(1).unwrap(); // 删除age列
    
    let options = WriteOptions::default();
    editor.save(&out, &options).unwrap();
    
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("name"));
    assert!(!content.contains("age"));
    assert!(content.contains("city"));
    assert!(content.contains("Alice"));
    assert!(content.contains("Beijing"));
    
    cleanup(&path);
    cleanup(&out);
}

#[test]
fn test_csv_creator() {
    let out = output_path();
    
    let headers = vec!["id".to_string(), "name".to_string()];
    let mut creator = CsvCreator::new(headers);
    
    creator.add_row(RowData::new(vec!["1".to_string(), "Alice".to_string()])).unwrap();
    creator.add_row(RowData::new(vec!["2".to_string(), "Bob".to_string()])).unwrap();
    
    let stats = creator.save(&out).unwrap();
    
    assert_eq!(stats.rows_written, 2);
    
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("id,name"));
    assert!(content.contains("1,Alice"));
    assert!(content.contains("2,Bob"));
    
    cleanup(&out);
}

#[test]
fn test_escape_special_chars() {
    let out = output_path();
    
    let headers = vec!["name".to_string(), "desc".to_string()];
    let mut creator = CsvCreator::new(headers);
    
    // 添加包含特殊字符的行
    creator.add_row(RowData::new(vec![
        "Alice".to_string(),
        "Hello, \"World\"".to_string(),
    ])).unwrap();
    
    creator.save(&out).unwrap();
    
    let content = fs::read_to_string(&out).unwrap();
    // 应该包含转义的引号
    assert!(content.contains("\"Hello, \"\"World\"\"\""));
    
    cleanup(&out);
}

#[test]
fn test_change_stats() {
    let content = "name,age\nAlice,25\nBob,30\n";
    let path = create_test_csv(content);
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    
    // 初始无修改
    assert!(!editor.has_changes());
    
    // 编辑后有修改
    editor.edit_cell(0, 0, "Updated".to_string()).unwrap();
    assert!(editor.has_changes());
    
    let stats = editor.change_stats();
    assert_eq!(stats.cells_edited, 1);
    
    // 清除修改
    editor.discard_changes();
    assert!(!editor.has_changes());
    
    cleanup(&path);
}

#[test]
fn test_set_header() {
    let content = "name,age\nAlice,25\n";
    let path = create_test_csv(content);
    let out = output_path();
    
    let mut editor = CsvEditor::open(&path, true, b',', 10).unwrap();
    editor.set_header(0, "full_name".to_string()).unwrap();
    
    let options = WriteOptions::default();
    editor.save(&out, &options).unwrap();
    
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("full_name"));
    assert!(!content.starts_with("name"));
    
    cleanup(&path);
    cleanup(&out);
}

#[test]
fn test_write_options() {
    let out = output_path();
    
    let headers = vec!["name".to_string(), "age".to_string()];
    let options = WriteOptions::new()
        .with_delimiter(b'\t')
        .with_always_quote(true);
    
    let mut creator = CsvCreator::new(headers).with_options(options);
    creator.add_row(RowData::new(vec!["Alice".to_string(), "25".to_string()])).unwrap();
    
    creator.save(&out).unwrap();
    
    let content = fs::read_to_string(&out).unwrap();
    // 使用制表符分隔
    assert!(content.contains("\t"));
    // 总是引用
    assert!(content.contains("\"name\""));
    
    cleanup(&out);
}

