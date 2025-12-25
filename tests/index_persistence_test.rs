use csv_tool::csv::{CsvReader, RowIndex, IndexMetadata};
use csv_tool::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

fn create_test_csv(path: &PathBuf, rows: usize) -> Result<()> {
    let mut file = File::create(path)?;
    
    // 写入表头
    writeln!(file, "id,name,age")?;
    
    // 写入数据行
    for i in 1..=rows {
        writeln!(file, "{},Name {},{}", i, i, 20 + i % 50)?;
    }
    
    Ok(())
}

#[test]
fn test_index_save_and_load() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_index_save.csv");
    create_test_csv(&test_file, 100)?;
    
    // 打开文件并构建索引
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let info = reader.info();
    
    // 检查索引文件是否已创建
    let index_path = RowIndex::index_file_path(&test_file);
    assert!(index_path.exists(), "索引文件应该已创建");
    
    // 再次打开文件，应该加载索引
    let start = std::time::Instant::now();
    let reader2 = CsvReader::open(&test_file, true, b',', 10)?;
    let load_duration = start.elapsed();
    
    // 验证数据正确性
    assert_eq!(reader2.info().total_rows, info.total_rows);
    assert_eq!(reader2.info().total_cols, info.total_cols);
    
    // 验证索引加载速度（应该很快）
    // 注意：在CI/并行测试环境中，时间可能会有波动，所以放宽限制
    println!("索引加载耗时: {:?}", load_duration);
    assert!(load_duration.as_millis() < 1000, "索引加载应该在1秒内完成");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    std::fs::remove_file(&index_path).ok();
    Ok(())
}

#[test]
fn test_index_invalid_after_file_modification() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_index_invalid.csv");
    create_test_csv(&test_file, 50)?;
    
    // 打开文件并构建索引
    let reader = CsvReader::open(&test_file, true, b',', 10)?;
    let initial_rows = reader.info().total_rows;
    
    // 修改CSV文件（添加一行）
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&test_file)?;
    writeln!(file, "101,New Name,30")?;
    drop(file);
    
    // 等待一下确保文件系统更新时间戳
    std::thread::sleep(Duration::from_millis(100));
    
    // 再次打开文件，索引应该失效并重建
    let reader2 = CsvReader::open(&test_file, true, b',', 10)?;
    let new_rows = reader2.info().total_rows;
    
    // 验证索引已重建（行数应该增加）
    assert_eq!(new_rows, initial_rows + 1, "索引应该已重建，行数应该增加");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    let index_path = RowIndex::index_file_path(&test_file);
    std::fs::remove_file(&index_path).ok();
    Ok(())
}

#[test]
fn test_index_metadata() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_metadata.csv");
    create_test_csv(&test_file, 50)?;
    
    let metadata = std::fs::metadata(&test_file)?;
    let file_size = metadata.len();
    let file_mtime = metadata.modified()?;
    
    let index_metadata = IndexMetadata::new(
        test_file.clone(),
        file_size,
        file_mtime,
        10,
    );
    
    assert_eq!(index_metadata.csv_size, file_size);
    assert_eq!(index_metadata.granularity, 10);
    assert_eq!(index_metadata.index_version, 1);
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    Ok(())
}

#[test]
fn test_index_file_path() {
    let csv_path = PathBuf::from("test.csv");
    let index_path = RowIndex::index_file_path(&csv_path);
    assert_eq!(index_path, PathBuf::from("test.csv.idx"));
    
    let csv_path2 = PathBuf::from("data/test.csv");
    let index_path2 = RowIndex::index_file_path(&csv_path2);
    assert_eq!(index_path2, PathBuf::from("data/test.csv.idx"));
}

