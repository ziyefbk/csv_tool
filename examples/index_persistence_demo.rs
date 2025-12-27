//! 索引持久化功能演示
//! 
//! 展示索引持久化如何提升重复打开文件的性能

use csv_tool::csv::CsvReader;
use csv_tool::error::Result;
use std::time::Instant;

fn main() -> Result<()> {
    println!("索引持久化功能演示\n");

    let csv_file = "examples/sample.csv";
    
    // 检查文件是否存在
    if !std::path::Path::new(csv_file).exists() {
        println!("示例文件不存在，请先运行 basic_usage 示例生成文件");
        return Ok(());
    }

    println!("📄 CSV文件: {}", csv_file);
    println!();

    // 第一次打开：构建索引
    println!("🔄 第一次打开文件（构建索引）...");
    let start1 = Instant::now();
    let reader1 = CsvReader::open(csv_file, true, b',', 100)?;
    let duration1 = start1.elapsed();
    let info1 = reader1.info();
    
    println!("✅ 打开完成");
    println!("   耗时: {:.2} 毫秒", duration1.as_secs_f64() * 1000.0);
    println!("   总行数: {}", info1.total_rows);
    println!("   总列数: {}", info1.total_cols);
    println!();

    // 检查索引文件是否已创建
    let index_path = csv_tool::csv::RowIndex::index_file_path(std::path::Path::new(csv_file));
    if index_path.exists() {
        let index_size = std::fs::metadata(&index_path)?.len();
        println!("💾 索引文件已创建: {}", index_path.display());
        println!("   索引文件大小: {:.2} KB", index_size as f64 / 1024.0);
        println!();
    }

    // 第二次打开：加载索引
    println!("🔄 第二次打开文件（加载索引）...");
    let start2 = Instant::now();
    let mut reader2 = CsvReader::open(csv_file, true, b',', 100)?;
    let duration2 = start2.elapsed();
    let info2 = reader2.info();
    
    println!("✅ 打开完成");
    println!("   耗时: {:.2} 毫秒", duration2.as_secs_f64() * 1000.0);
    println!("   总行数: {}", info2.total_rows);
    println!("   总列数: {}", info2.total_cols);
    println!();

    // 性能对比
    let speedup = duration1.as_secs_f64() / duration2.as_secs_f64();
    println!("📊 性能对比:");
    println!("   首次打开: {:.2} 毫秒", duration1.as_secs_f64() * 1000.0);
    println!("   再次打开: {:.2} 毫秒", duration2.as_secs_f64() * 1000.0);
    println!("   性能提升: {:.1}x", speedup);
    println!();

    // 读取数据验证正确性
    println!("📖 读取第一页数据验证...");
    let rows = reader2.read_page(0, 5)?;
    println!("   读取了 {} 行数据", rows.len());
    for (i, row) in rows.iter().take(3).enumerate() {
        println!("   行 {}: {:?}", i + 1, row.fields.iter().take(3).map(|f| f.as_ref()).collect::<Vec<_>>());
    }
    println!();

    println!("✨ 索引持久化功能正常工作！");
    println!("💡 提示: 索引文件保存在: {}", index_path.display());
    println!("   删除索引文件后，下次打开会重新构建索引");

    Ok(())
}

