use csv::ReaderBuilder;
use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

const PAGE_SIZE: usize = 20; // 每页显示行数

struct CsvInfo {
    file_path: String,
    file_size: u64,
    total_rows: usize,
    total_cols: usize,
    headers: Vec<String>,
}

fn get_file_info(file_path: &str) -> Result<CsvInfo, Box<dyn Error>> {
    let metadata = fs::metadata(file_path)?;
    let file_size = metadata.len();
    
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    
    let headers: Vec<String> = reader.headers()?.iter().map(|s| s.to_string()).collect();
    let total_cols = headers.len();
    
    // 计算总行数（不包括表头）
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    let total_rows = buf_reader.lines().count().saturating_sub(1);
    
    Ok(CsvInfo {
        file_path: file_path.to_string(),
        file_size,
        total_rows,
        total_cols,
        headers,
    })
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn read_page(file_path: &str, page: usize, page_size: usize) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    
    let skip = page * page_size;
    let mut rows = Vec::new();
    
    for (i, result) in reader.records().enumerate() {
        if i < skip {
            continue;
        }
        if i >= skip + page_size {
            break;
        }
        let record = result?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }
    
    Ok(rows)
}

fn print_table(headers: &[String], rows: &[Vec<String>], page: usize, total_pages: usize) {
    println!("\n{}", "═".repeat(100));
    
    // 打印表头
    print!("│ ");
    for header in headers {
        print!("{:15} │ ", truncate_str(header, 15));
    }
    println!();
    println!("{}", "─".repeat(100));
    
    // 打印数据行
    for row in rows {
        print!("│ ");
        for field in row {
            print!("{:15} │ ", truncate_str(field, 15));
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
    println!("CSV文件查看工具 v0.2.0");
    println!("\n用法: {} <文件路径> [页码]", program);
    println!("\n参数:");
    println!("  <文件路径>  CSV文件的路径");
    println!("  [页码]      可选，指定要显示的页码（从1开始）");
    println!("\n示例:");
    println!("  {} test.csv       # 显示第1页", program);
    println!("  {} test.csv 2     # 显示第2页", program);
    println!("\n每页显示 {} 行数据", PAGE_SIZE);
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help(&args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    
    // 获取文件信息
    let info = get_file_info(file_path)?;
    let total_pages = (info.total_rows + PAGE_SIZE - 1) / PAGE_SIZE;
    
    // 解析页码参数
    let page: usize = if args.len() >= 3 {
        args[2].parse::<usize>().unwrap_or(1).saturating_sub(1)
    } else {
        0
    };
    
    let page = page.min(total_pages.saturating_sub(1));
    
    // 打印文件信息
    println!("\n📄 文件: {}", info.file_path);
    println!("📊 大小: {}", format_size(info.file_size));
    println!("📋 总行数: {} 行", info.total_rows);
    println!("📑 总列数: {} 列", info.total_cols);
    println!("📖 总页数: {} 页（每页 {} 行）", total_pages, PAGE_SIZE);
    
    // 读取并显示指定页
    let rows = read_page(file_path, page, PAGE_SIZE)?;
    print_table(&info.headers, &rows, page, total_pages);
    
    if total_pages > 1 {
        println!("\n💡 提示: 使用 '{} {} <页码>' 查看其他页", args[0], file_path);
    }
    
    Ok(())
}
