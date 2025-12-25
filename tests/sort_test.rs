//! 排序功能集成测试

use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use csv_tool::csv::{
    CsvReader, SortOrder, SortKey, SortOptions, DataType, sort_csv_data
};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 创建临时测试文件（使用唯一计数器避免冲突）
fn create_test_csv(content: &str) -> String {
    let counter = TEST_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
    let path = format!("target/test_sort_{}_{}.csv", std::process::id(), counter);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// 清理测试文件
fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_sort_string_ascending() {
    let content = "name,value\nCharlie,3\nAlice,1\nBob,2\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(0, SortOrder::Ascending, DataType::String);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Alice");
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Bob");
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Charlie");
    
    cleanup(&path);
}

#[test]
fn test_sort_string_descending() {
    let content = "name,value\nAlice,1\nBob,2\nCharlie,3\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(0, SortOrder::Descending, DataType::String);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Charlie");
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Bob");
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Alice");
    
    cleanup(&path);
}

#[test]
fn test_sort_number_ascending() {
    let content = "name,score\nAlice,95\nBob,85\nCharlie,90\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(1, SortOrder::Ascending, DataType::Number);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Bob");    // 85
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Charlie"); // 90
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Alice");   // 95
    
    cleanup(&path);
}

#[test]
fn test_sort_number_descending() {
    let content = "name,score\nAlice,95\nBob,85\nCharlie,90\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(1, SortOrder::Descending, DataType::Number);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    assert_eq!(sorted.len(), 3);
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Alice");   // 95
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Charlie"); // 90
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Bob");     // 85
    
    cleanup(&path);
}

#[test]
fn test_sort_auto_detection() {
    // Auto类型应该能自动识别数字并正确排序
    let content = "name,value\nA,10\nB,2\nC,100\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(1, SortOrder::Ascending, DataType::Auto);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    // 数字应该按数值排序：2 < 10 < 100
    assert_eq!(sorted[0].record.fields[0].as_ref(), "B");   // 2
    assert_eq!(sorted[1].record.fields[0].as_ref(), "A");   // 10
    assert_eq!(sorted[2].record.fields[0].as_ref(), "C");   // 100
    
    cleanup(&path);
}

#[test]
fn test_sort_case_insensitive() {
    let content = "name,value\nAlice,1\nalice,2\nBob,3\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(0, SortOrder::Ascending, DataType::String);
    let options = SortOptions::new()
        .add_key(key)
        .with_case_sensitive(false);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    // Alice和alice应该相邻（不区分大小写）
    assert_eq!(sorted.len(), 3);
    let first_two: Vec<&str> = sorted[0..2].iter()
        .map(|r| r.record.fields[0].as_ref())
        .collect();
    assert!(first_two.contains(&"Alice") && first_two.contains(&"alice"));
    
    cleanup(&path);
}

#[test]
fn test_sort_with_limit() {
    let content = "name,score\nAlice,95\nBob,85\nCharlie,90\nDavid,88\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(1, SortOrder::Descending, DataType::Number);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, Some(2)).unwrap();
    
    // 只返回前2个
    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Alice");   // 95
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Charlie"); // 90
    
    cleanup(&path);
}

#[test]
fn test_sort_preserves_original_row_numbers() {
    let content = "name,value\nCharlie,3\nAlice,1\nBob,2\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::ascending(0).with_data_type(DataType::String);
    let options = SortOptions::new().add_key(key);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    // Alice原本在第2行（索引1）
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Alice");
    assert_eq!(sorted[0].original_row, 1);
    
    // Bob原本在第3行（索引2）
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Bob");
    assert_eq!(sorted[1].original_row, 2);
    
    // Charlie原本在第1行（索引0）
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Charlie");
    assert_eq!(sorted[2].original_row, 0);
    
    cleanup(&path);
}

#[test]
fn test_sort_empty_values() {
    let content = "name,score\nAlice,95\nBob,\nCharlie,90\n";
    let path = create_test_csv(content);
    
    let reader = CsvReader::open(&path, true, b',', 10).unwrap();
    
    let key = SortKey::new(1, SortOrder::Descending, DataType::Number);
    let options = SortOptions::new()
        .add_key(key)
        .with_nulls_last(true);
    
    let sorted = sort_csv_data(&reader, &options, None).unwrap();
    
    // 空值应该排在最后
    assert_eq!(sorted[0].record.fields[0].as_ref(), "Alice");   // 95
    assert_eq!(sorted[1].record.fields[0].as_ref(), "Charlie"); // 90
    assert_eq!(sorted[2].record.fields[0].as_ref(), "Bob");     // 空
    
    cleanup(&path);
}

