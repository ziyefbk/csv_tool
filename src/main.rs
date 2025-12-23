use csv_tool::csv::{CsvReader, format_size};
use csv_tool::error::Result;
use std::env;

const PAGE_SIZE: usize = 20; // 每页显示行数

fn print_table(headers: &[String], rows: &[csv_tool::csv::CsvRecord], page: usize, total_pages: usize) {
    println!("\n{}", "═".repeat(100));
    
    // 打印表头
    if !headers.is_empty() {
        print!("│ ");
        for header in headers {
            print!("{:15} │ ", truncate_str(header, 15));
        }
        println!();
        println!("{}", "─".repeat(100));
    }
    
    // 打印数据行
    for row in rows {
        print!("│ ");
        for field in &row.fields {
            let field_str = field.as_ref();
            print!("{:15} │ ", truncate_str(field_str, 15));
        }
        println!();
    }
    
    println!("{}", "═".repeat(100));
    println!("第 {}/{} 页", page + 1, total_pages);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 2).collect();
        format!("{}..", truncated)
    } else {
        s.to_string()
    }
}

fn print_help(program: &str) {
    println!("CSV文件查看工具 v0.3.0 (高性能版本)");
    println!("\n用法: {} <文件路径> [页码]", program);
    println!("\n参数:");
    println!("  <文件路径>  CSV文件的路径");
    println!("  [页码]      可选，指定要显示的页码（从1开始）");
    println!("\n示例:");
    println!("  {} test.csv       # 显示第1页", program);
    println!("  {} test.csv 2     # 显示第2页", program);
    println!("\n每页显示 {} 行数据", PAGE_SIZE);
    println!("\n特性:");
    println!("  ✨ 使用内存映射技术，支持GB级大文件");
    println!("  ⚡ 稀疏行索引，快速页面跳转");
    println!("  💾 页面缓存，提升重复访问性能");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help(&args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    
    println!("\n🔄 正在打开文件: {}...", file_path);
    let start_time = std::time::Instant::now();
    
    // 使用新的高性能读取器
    let mut reader = CsvReader::open(
        file_path,
        true,  // 假设有表头
        b',',  // 逗号分隔符
        1000,  // 每1000行记录一次索引
    )?;
    
    let open_duration = start_time.elapsed();
    
    // 先获取文件信息（克隆以避免借用冲突）
    let info = reader.info().clone();
    let total_pages = reader.total_pages(PAGE_SIZE);
    
    // 解析页码参数
    let page: usize = if args.len() >= 3 {
        args[2].parse::<usize>().unwrap_or(1).saturating_sub(1)
    } else {
        0
    };
    
    let page = page.min(total_pages.saturating_sub(1));
    
    // 打印文件信息
    println!("\n📄 文件: {}", info.file_path.display());
    println!("📊 大小: {}", format_size(info.file_size));
    println!("📋 总行数: {} 行", info.total_rows);
    println!("📑 总列数: {} 列", info.total_cols);
    println!("📖 总页数: {} 页（每页 {} 行）", total_pages, PAGE_SIZE);
    println!("⏱️  打开耗时: {:.2}秒", open_duration.as_secs_f64());
    
    // 读取并显示指定页
    let read_start = std::time::Instant::now();
    let rows = reader.read_page(page, PAGE_SIZE)?;
    let read_duration = read_start.elapsed();
    
    println!("⚡ 读取耗时: {:.2}毫秒", read_duration.as_secs_f64() * 1000.0);
    
    print_table(&info.headers, &rows, page, total_pages);
    
    if total_pages > 1 {
        println!("\n💡 提示: 使用 '{} {} <页码>' 查看其他页", args[0], file_path);
    }
    
    Ok(())
}
