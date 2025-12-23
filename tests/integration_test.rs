use csv_tool::csv::CsvReader;
use csv_tool::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn create_test_csv(path: &PathBuf, rows: usize) -> Result<()> {
    let mut file = File::create(path)?;
    
    // 写入表头
    writeln!(file, "id,name,age,city")?;
    
    // 写入数据行
    for i in 1..=rows {
        writeln!(file, "{},\"Name {}\",{},City {}", i, i, 20 + i % 50, i % 10)?;
    }
    
    Ok(())
}

#[test]
fn test_basic_read() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_basic.csv");
    create_test_csv(&test_file, 100)?;
    
    let mut reader = CsvReader::open(&test_file, true, b',', 10)?;
    let info = reader.info();
    
    assert_eq!(info.total_rows, 100);
    assert_eq!(info.total_cols, 4);
    assert_eq!(info.headers.len(), 4);
    
    // 读取第一页
    let rows = reader.read_page(0, 20)?;
    assert_eq!(rows.len(), 20);
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    Ok(())
}

#[test]
fn test_page_access() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_pages.csv");
    create_test_csv(&test_file, 100)?;
    
    let mut reader = CsvReader::open(&test_file, true, b',', 10)?;
    
    // 读取第0页
    let page0 = reader.read_page(0, 20)?;
    assert_eq!(page0.len(), 20);
    
    // 读取第1页
    let page1 = reader.read_page(1, 20)?;
    assert_eq!(page1.len(), 20);
    
    // 读取最后一页
    let last_page = reader.read_page(4, 20)?;
    assert_eq!(last_page.len(), 20);
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    Ok(())
}

#[test]
fn test_index_seek() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_index.csv");
    create_test_csv(&test_file, 1000)?;
    
    let mut reader = CsvReader::open(&test_file, true, b',', 100)?;
    
    // 测试跳转到中间页面
    let page = reader.read_page(25, 20)?;
    assert_eq!(page.len(), 20);
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    Ok(())
}

#[test]
fn test_quoted_fields() -> Result<()> {
    let test_file = std::env::temp_dir().join("test_quoted.csv");
    let mut file = File::create(&test_file)?;
    
    writeln!(file, "col1,col2,col3")?;
    writeln!(file, "\"quoted,field\",normal,\"another\"\"quote\"\"\"")?;
    
    let mut reader = CsvReader::open(&test_file, true, b',', 10)?;
    let rows = reader.read_page(0, 10)?;
    
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].fields.len(), 3);
    
    // 检查字段内容（注意字段顺序）
    let field0 = rows[0].fields[0].as_ref();
    let field1 = rows[0].fields[1].as_ref();
    let field2 = rows[0].fields[2].as_ref();
    
    // 第一个字段应该是 "quoted,field"
    assert_eq!(field0, "quoted,field", "第一个字段应该是 'quoted,field'，但得到 '{}'", field0);
    // 第二个字段应该是 "normal"
    assert_eq!(field1, "normal");
    // 第三个字段应该是 "another\"quote\""
    assert_eq!(field2, "another\"quote\"");
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    Ok(())
}

