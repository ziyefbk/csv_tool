//! 搜索功能集成测试

use csv_tool::csv::{CsvReader, SearchPattern, SearchOptions};
use csv_tool::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn create_test_csv(path: &PathBuf) -> Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, "id,name,email,city")?;
    writeln!(file, "1,Alice,alice@example.com,Beijing")?;
    writeln!(file, "2,Bob,bob@test.org,Shanghai")?;
    writeln!(file, "3,Charlie,charlie@example.com,Guangzhou")?;
    writeln!(file, "4,David,david@test.org,Shenzhen")?;
    writeln!(file, "5,Eve,eve@example.com,Beijing")?;
    
    Ok(())
}

#[test]
fn test_text_search() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_text.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 搜索 "Beijing"
    let pattern = SearchPattern::text("Beijing", true);
    let options = SearchOptions::new(pattern);
    let results = reader.search(&options)?;
    
    assert_eq!(results.len(), 2, "应该找到2个包含Beijing的行");
    assert_eq!(results[0].row_number, 0);
    assert_eq!(results[1].row_number, 4);
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_regex_search() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_regex.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 使用正则表达式搜索邮箱域名
    let pattern = SearchPattern::regex(r"@example\.com", true)?;
    let options = SearchOptions::new(pattern);
    let results = reader.search(&options)?;
    
    assert_eq!(results.len(), 3, "应该找到3个example.com邮箱");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_search_in_column() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_column.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 只在name列(索引1)中搜索
    let pattern = SearchPattern::text("e", false);  // 大小写不敏感
    let options = SearchOptions::new(pattern)
        .with_columns(vec![1])  // name列
        .with_case_sensitive(false);
    let results = reader.search(&options)?;
    
    // Alice, Charlie, Eve 包含 'e'
    assert_eq!(results.len(), 3, "name列中应该有3个包含'e'的名字");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_search_case_insensitive() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_case.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 大小写不敏感搜索
    let pattern = SearchPattern::text("BEIJING", false);
    let options = SearchOptions::new(pattern)
        .with_case_sensitive(false);
    let results = reader.search(&options)?;
    
    assert_eq!(results.len(), 2, "大小写不敏感时应该找到2个Beijing");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_search_invert_match() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_invert.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 反向匹配：找出不包含Beijing的行
    let pattern = SearchPattern::text("Beijing", true);
    let options = SearchOptions::new(pattern)
        .with_invert_match(true);
    let results = reader.search(&options)?;
    
    assert_eq!(results.len(), 3, "应该有3行不包含Beijing");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_count_matches() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_count.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    let pattern = SearchPattern::text("example", true);
    let options = SearchOptions::new(pattern);
    let count = reader.count_matches(&options)?;
    
    assert_eq!(count, 3, "应该有3行包含example");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}

#[test]
fn test_search_max_results() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_search_max.csv");
    create_test_csv(&test_file)?;
    
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 限制最大结果数为2
    let pattern = SearchPattern::text("example", true);
    let options = SearchOptions::new(pattern)
        .with_max_results(2);
    let results = reader.search(&options)?;
    
    assert_eq!(results.len(), 2, "应该最多返回2个结果");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = csv_tool::csv::RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    
    Ok(())
}


