//! CSV工具基本使用示例
//! 
//! 展示如何使用csv_tool库进行CSV文件读取和处理

use csv_tool::csv::CsvReader;
use csv_tool::error::Result;

fn main() -> Result<()> {
    println!("CSV工具使用示例\n");

    // 示例1: 打开CSV文件
    println!("示例1: 打开CSV文件");
    let mut reader = CsvReader::open(
        "examples/sample.csv",
        true,   // 有表头
        b',',   // 逗号分隔符
        100,    // 索引粒度（每100行记录一次）
    )?;

    // 获取文件信息
    let info = reader.info();
    println!("文件: {}", info.file_path.display());
    println!("大小: {} 字节", info.file_size);
    println!("总行数: {}", info.total_rows);
    println!("总列数: {}", info.total_cols);
    println!("表头: {:?}\n", info.headers);

    // 示例2: 读取第一页
    println!("示例2: 读取第一页（每页10行）");
    let page_size = 10;
    let rows = reader.read_page(0, page_size)?;
    
    println!("读取了 {} 行数据:", rows.len());
    for (i, row) in rows.iter().enumerate() {
        print!("行 {}: ", i + 1);
        for field in &row.fields {
            print!("{} | ", field.as_ref());
        }
        println!();
    }
    println!();

    // 示例3: 计算总页数并读取不同页面
    println!("示例3: 分页浏览");
    let total_pages = reader.total_pages(page_size);
    println!("总页数: {}", total_pages);
    
    if total_pages > 1 {
        println!("读取第2页:");
        let page2 = reader.read_page(1, page_size)?;
        println!("第2页有 {} 行数据\n", page2.len());
    }

    // 示例4: 性能测试
    println!("示例4: 性能测试");
    use std::time::Instant;
    
    let start = Instant::now();
    let _rows = reader.read_page(0, page_size)?;
    let duration = start.elapsed();
    
    println!("读取第1页耗时: {:.2} 毫秒", duration.as_secs_f64() * 1000.0);
    
    if total_pages > 10 {
        let start = Instant::now();
        let _rows = reader.read_page(10, page_size)?;
        let duration = start.elapsed();
        println!("跳转到第11页耗时: {:.2} 毫秒", duration.as_secs_f64() * 1000.0);
    }

    Ok(())
}

