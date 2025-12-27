use clap::{Parser, Subcommand};
use csv_tool::csv::{CsvReader, RowIndex, format_size, SearchPattern, SearchOptions, highlight_matches, ExportFormat, ExportOptions, Exporter, SortOrder, SortKey, SortOptions, DataType, sort_csv_data, CsvEditor, CsvCreator, RowData, WriteOptions};
use csv_tool::error::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Instant;

/// 高性能CSV文件查看工具
#[derive(Parser)]
#[command(name = "csv-tool")]
#[command(author, version, about, long_about = "
CSV Tool - 高性能CSV文件查看和处理工具

特性:
  ✨ 使用内存映射技术，支持GB级大文件
  ⚡ 稀疏行索引，快速页面跳转
  💾 索引持久化，重复打开更快
  🔄 LRU页面缓存，提升访问性能
  🔍 全文搜索和正则表达式支持

示例:
  csv-tool data.csv              查看第1页
  csv-tool data.csv -p 5         查看第5页
  csv-tool data.csv info         显示文件详细信息
  csv-tool data.csv search 关键词  搜索关键词
  csv-tool data.csv -d ';'       使用分号作为分隔符
")]
struct Args {
    /// CSV文件路径
    #[arg(value_name = "FILE")]
    file: String,

    /// 页码（从1开始，向后兼容：可直接传递数字而不使用-p）
    #[arg(value_name = "PAGE", help_heading = "向后兼容")]
    page_arg: Option<usize>,

    /// 分隔符字符
    #[arg(short, long, default_value = ",", value_name = "CHAR")]
    delimiter: char,

    /// 页码（从1开始）
    #[arg(short, long, value_name = "PAGE")]
    page: Option<usize>,

    /// 每页显示行数
    #[arg(short = 's', long, default_value = "20", value_name = "SIZE")]
    page_size: usize,

    /// 文件不包含表头
    #[arg(short = 'n', long)]
    no_headers: bool,

    /// 索引粒度（每N行记录一次索引点）
    #[arg(short, long, default_value = "1000", value_name = "N")]
    granularity: usize,

    /// 安静模式（减少输出信息）
    #[arg(short, long)]
    quiet: bool,

    /// 详细模式（显示更多信息）
    #[arg(short, long)]
    verbose: bool,

    /// 强制重建索引（忽略缓存的索引文件）
    #[arg(long)]
    rebuild_index: bool,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 显示文件详细信息
    Info,
    
    /// 查看CSV数据（默认行为）
    View {
        /// 指定查看的页码
        #[arg(short, long)]
        page: Option<usize>,
    },

    /// 搜索CSV数据
    Search {
        /// 搜索模式（文本或正则表达式）
        pattern: String,

        /// 使用正则表达式
        #[arg(short = 'r', long)]
        regex: bool,

        /// 大小写不敏感
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// 在指定列中搜索（列名或列号，从1开始）
        #[arg(short = 'c', long, value_name = "COLUMN")]
        column: Option<String>,

        /// 显示行号
        #[arg(short = 'l', long)]
        line_numbers: bool,

        /// 只统计匹配数量
        #[arg(long)]
        count: bool,

        /// 最大结果数
        #[arg(short = 'm', long, value_name = "N")]
        max_results: Option<usize>,

        /// 反向匹配（显示不匹配的行）
        #[arg(short = 'V', long)]
        invert_match: bool,

        /// 禁用高亮显示
        #[arg(long)]
        no_highlight: bool,
    },

    /// 导出CSV数据为其他格式
    Export {
        /// 输出文件路径
        output: String,

        /// 导出格式 (json, jsonl, csv, tsv)
        #[arg(short, long, value_name = "FORMAT")]
        format: Option<String>,

        /// 导出指定列（列名或列号，逗号分隔）
        #[arg(short = 'c', long, value_name = "COLUMNS")]
        columns: Option<String>,

        /// 起始行（从1开始）
        #[arg(long, value_name = "ROW")]
        from: Option<usize>,

        /// 结束行
        #[arg(long, value_name = "ROW")]
        to: Option<usize>,

        /// 只导出匹配搜索条件的行
        #[arg(long, value_name = "PATTERN")]
        search: Option<String>,

        /// 搜索使用正则表达式
        #[arg(short = 'r', long)]
        regex: bool,

        /// JSON美化输出
        #[arg(long)]
        pretty: bool,

        /// 不包含表头（CSV/TSV格式）
        #[arg(long)]
        no_headers: bool,
    },

    /// 按列排序数据
    Sort {
        /// 排序列（列名或列号，从1开始）
        #[arg(value_name = "COLUMN")]
        column: String,

        /// 排序方向 (asc/desc)
        #[arg(long, default_value = "asc")]
        order: String,

        /// 数据类型 (auto/string/number)
        #[arg(short = 't', long, default_value = "auto")]
        data_type: String,

        /// 显示结果数量限制
        #[arg(short = 'n', long, value_name = "N")]
        limit: Option<usize>,

        /// 大小写不敏感（字符串排序）
        #[arg(short = 'i', long)]
        ignore_case: bool,

        /// 空值排在最前
        #[arg(long)]
        nulls_first: bool,

        /// 显示行号
        #[arg(short = 'l', long)]
        line_numbers: bool,

        /// 导出排序结果到文件
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<String>,
    },

    /// 编辑CSV文件
    Edit {
        /// 编辑操作类型
        #[command(subcommand)]
        action: EditAction,
    },

    /// 创建新的CSV文件
    Create {
        /// 输出文件路径
        output: String,

        /// 表头列表（逗号分隔）
        #[arg(short = 'H', long, value_name = "HEADERS")]
        headers: String,

        /// 数据行（逗号分隔，可多次使用）
        #[arg(short = 'r', long = "row", value_name = "ROW")]
        rows: Vec<String>,
    },
}

/// 编辑操作
#[derive(Subcommand, Clone)]
enum EditAction {
    /// 修改单元格值
    Cell {
        /// 行号（从1开始）
        #[arg(short, long)]
        row: usize,

        /// 列（列名或列号）
        #[arg(short, long)]
        col: String,

        /// 新值
        #[arg(short, long)]
        value: String,

        /// 输出文件路径（不指定则覆盖原文件）
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 删除行
    DeleteRow {
        /// 要删除的行号（从1开始，可多个，逗号分隔）
        #[arg(short, long, value_name = "ROWS")]
        rows: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 添加新行
    AddRow {
        /// 新行数据（逗号分隔）
        #[arg(short = 'd', long, value_name = "DATA")]
        data: String,

        /// 插入位置（行号，从1开始，不指定则追加到末尾）
        #[arg(short = 'p', long, value_name = "POSITION")]
        position: Option<usize>,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 删除列
    DeleteCol {
        /// 要删除的列（列名或列号，可多个，逗号分隔）
        #[arg(short, long, value_name = "COLS")]
        cols: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 重命名列
    RenameCol {
        /// 原列名或列号
        #[arg(short, long)]
        col: String,

        /// 新列名
        #[arg(short, long)]
        name: String,

        /// 输出文件路径
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // 向后兼容：如果直接传递了页码数字（page_arg），优先使用它
    let final_page = if let Some(page_arg) = args.page_arg {
        page_arg
    } else {
        args.page.unwrap_or(1)
    };
    
    match &args.command {
        Some(Commands::Info) => cmd_info(&args),
        Some(Commands::View { page }) => {
            let page_num = page.or(Some(final_page)).unwrap_or(1);
            cmd_view(&args, page_num)
        }
        Some(Commands::Search { 
            pattern, 
            regex, 
            ignore_case, 
            column, 
            line_numbers, 
            count, 
            max_results,
            invert_match,
            no_highlight,
        }) => cmd_search(
            &args, 
            pattern, 
            *regex, 
            *ignore_case, 
            column.as_deref(), 
            *line_numbers, 
            *count, 
            *max_results,
            *invert_match,
            *no_highlight,
        ),
        Some(Commands::Export {
            output,
            format,
            columns,
            from,
            to,
            search,
            regex,
            pretty,
            no_headers,
        }) => cmd_export(
            &args,
            output,
            format.as_deref(),
            columns.as_deref(),
            *from,
            *to,
            search.as_deref(),
            *regex,
            *pretty,
            *no_headers,
        ),
        Some(Commands::Sort {
            column,
            order,
            data_type,
            limit,
            ignore_case,
            nulls_first,
            line_numbers,
            output,
        }) => cmd_sort(
            &args,
            column,
            order,
            data_type,
            *limit,
            *ignore_case,
            *nulls_first,
            *line_numbers,
            output.as_deref(),
        ),
        Some(Commands::Edit { action }) => cmd_edit(&args, action),
        Some(Commands::Create { output, headers, rows }) => cmd_create(
            output,
        headers,
            rows,
            args.delimiter as u8,
        ),
        None => cmd_view(&args, final_page),
    }
}

/// 显示文件详细信息
fn cmd_info(args: &Args) -> Result<()> {
    let start_time = Instant::now();
    
    // 显示加载提示
    if !args.quiet {
        println!("\n🔄 正在分析文件: {}...", args.file);
    }
    
    let pb = create_spinner("正在打开文件...");
    
    let reader = CsvReader::open_fast(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    pb.finish_and_clear();
    
    let info = reader.info();
    let open_duration = start_time.elapsed();
    
    // 检查索引文件
    let index_path = RowIndex::index_file_path(std::path::Path::new(&args.file));
    let index_exists = index_path.exists();
    let index_size = if index_exists {
        std::fs::metadata(&index_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    📄 CSV 文件信息                           ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 文件路径: {:<50} ║", truncate_path(&args.file, 50));
    println!("║ 文件大小: {:<50} ║", format_size(info.file_size));
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 总行数:   {:<50} ║", format!("{} 行", info.total_rows));
    println!("║ 总列数:   {:<50} ║", format!("{} 列", info.total_cols));
    println!("║ 有表头:   {:<50} ║", if !args.no_headers { "是" } else { "否" });
    println!("║ 分隔符:   {:<50} ║", format!("'{}'", args.delimiter));
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 索引缓存: {:<50} ║", if index_exists { 
        format!("✅ 存在 ({})", format_size(index_size)) 
    } else { 
        "❌ 无".to_string() 
    });
    println!("║ 索引粒度: {:<50} ║", format!("每 {} 行", args.granularity));
    println!("║ 分析耗时: {:<50} ║", format!("{:.2} 秒", open_duration.as_secs_f64()));
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    // 表头信息
    if !args.no_headers && !info.headers.is_empty() {
        println!("\n📋 表头列名:");
        for (i, header) in info.headers.iter().enumerate() {
            println!("   {}. {}", i + 1, header);
        }
    }
    
    if args.verbose {
        println!("\n📊 详细统计:");
        println!("   索引点数量: {}", info.total_rows / args.granularity);
        println!("   页面数量: {} (每页 {} 行)", 
            (info.total_rows + args.page_size - 1) / args.page_size,
            args.page_size
        );
        if index_exists {
            println!("   索引文件: {}", index_path.display());
        }
    }
    
    Ok(())
}

/// 查看CSV数据
fn cmd_view(args: &Args, page: usize) -> Result<()> {
    let start_time = Instant::now();
    
    // 显示加载提示
    if !args.quiet {
        println!("\n🔄 正在打开文件: {}...", args.file);
    }
    
    // 检查是否需要构建索引
    let index_path = RowIndex::index_file_path(std::path::Path::new(&args.file));
    let needs_build = !index_path.exists();
    
    let pb = if needs_build {
        // 需要构建索引，显示进度条
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("正在构建索引...");
        Some(pb)
    } else {
        // 只需要加载索引，显示spinner
        Some(create_spinner("正在加载索引..."))
    };
    
    let mut reader = CsvReader::open_fast(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    if let Some(pb) = pb {
        pb.finish_and_clear();
    }
    
    let open_duration = start_time.elapsed();
    
    // 如果是第一次构建索引，显示提示信息
    if needs_build && !args.quiet {
        println!("💡 提示: 索引已构建并保存，下次打开会更快！");
    }
    
    // 获取文件信息
    let info = reader.info().clone();
    let total_pages = reader.total_pages(args.page_size);
    
    // 调整页码（用户输入从1开始，内部从0开始）
    let page_idx = page.saturating_sub(1).min(total_pages.saturating_sub(1));
    
    // 打印文件信息（非安静模式）
    if !args.quiet {
        println!("\n📄 文件: {}", info.file_path.display());
        println!("📊 大小: {} | 📋 {} 行 × {} 列 | 📖 {} 页",
            format_size(info.file_size),
            info.total_rows,
            info.total_cols,
            total_pages
        );
        println!("⏱️  加载耗时: {:.2}秒", open_duration.as_secs_f64());
    }
    
    // 读取指定页
    let read_start = Instant::now();
    let rows = reader.read_page(page_idx, args.page_size)?;
    let read_duration = read_start.elapsed();
    
    if !args.quiet {
        println!("⚡ 读取耗时: {:.2}毫秒", read_duration.as_secs_f64() * 1000.0);
    }
    
    // 打印表格
    print_table(&info.headers, &rows, page_idx, total_pages, args.page_size);
    
    // 导航提示
    if !args.quiet && total_pages > 1 {
        println!("\n💡 导航提示:");
        if page_idx > 0 {
            println!("   上一页: csv-tool {} -p {}", args.file, page_idx);
        }
        if page_idx < total_pages - 1 {
            println!("   下一页: csv-tool {} -p {}", args.file, page_idx + 2);
        }
        println!("   跳转到: csv-tool {} -p <页码>", args.file);
    }
    
    Ok(())
}

/// 搜索CSV数据
fn cmd_search(
    args: &Args,
    pattern: &str,
    use_regex: bool,
    ignore_case: bool,
    column: Option<&str>,
    show_line_numbers: bool,
    count_only: bool,
    max_results: Option<usize>,
    invert_match: bool,
    no_highlight: bool,
) -> Result<()> {
    let start_time = Instant::now();
    
    if !args.quiet {
        println!("\n🔍 搜索模式: {}", if use_regex { "正则表达式" } else { "文本" });
        println!("📝 搜索内容: \"{}\"", pattern);
        if ignore_case {
            println!("🔤 大小写: 不敏感");
        }
        if invert_match {
            println!("🔄 模式: 反向匹配（显示不匹配的行）");
        }
    }
    
    let pb = create_spinner("正在打开文件...");
    
    let reader = CsvReader::open_fast(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    pb.finish_and_clear();
    
    let info = reader.info();
    let headers = info.headers.clone();
    
    // 解析目标列
    let target_columns = if let Some(col_str) = column {
        let col_idx = parse_column_spec(col_str, &headers)?;
        Some(vec![col_idx])
    } else {
        None
    };
    
    // 创建搜索模式
    let search_pattern = if use_regex {
        SearchPattern::regex(pattern, !ignore_case)?
    } else {
        SearchPattern::text(pattern, !ignore_case)
    };
    
    // 创建搜索选项
    let mut options = SearchOptions::new(search_pattern)
        .with_case_sensitive(!ignore_case)
        .with_invert_match(invert_match);
    
    if let Some(cols) = target_columns {
        options = options.with_columns(cols);
    }
    
    if let Some(max) = max_results {
        options = options.with_max_results(max);
    }
    
    // 执行搜索
    let search_start = Instant::now();
    
    if count_only {
        // 只统计数量
        let pb = create_spinner("正在搜索...");
        let count = reader.count_matches(&options)?;
        pb.finish_and_clear();
        
        let search_duration = search_start.elapsed();
        
        println!("\n📊 搜索结果统计:");
        println!("   匹配行数: {}", count);
        println!("   总行数:   {}", info.total_rows);
        println!("   匹配率:   {:.2}%", (count as f64 / info.total_rows as f64) * 100.0);
        println!("   搜索耗时: {:.2}毫秒", search_duration.as_secs_f64() * 1000.0);
    } else {
        // 返回详细结果
        let pb = create_spinner("正在搜索...");
        let results = reader.search(&options)?;
        pb.finish_and_clear();
        
        let search_duration = search_start.elapsed();
        let result_count = results.len();
        
        if !args.quiet {
            println!("\n✅ 找到 {} 个匹配", result_count);
            println!("⏱️  搜索耗时: {:.2}毫秒\n", search_duration.as_secs_f64() * 1000.0);
        }
        
        if result_count == 0 {
            println!("❌ 未找到匹配的结果");
            return Ok(());
        }
        
        // 打印搜索结果
        print_search_results(
            &results, 
            &headers, 
            show_line_numbers, 
            !no_highlight && !invert_match,
            args.page_size.min(result_count),
        );
        
        // 显示更多提示
        if result_count > args.page_size {
            println!("\n💡 显示了前 {} 条结果，共 {} 条匹配", 
                args.page_size.min(result_count), 
                result_count
            );
            println!("   使用 -m <N> 参数限制结果数量");
        }
    }
    
    let total_duration = start_time.elapsed();
    if args.verbose {
        println!("\n📊 性能统计:");
        println!("   总耗时: {:.2}秒", total_duration.as_secs_f64());
    }
    
    Ok(())
}

/// 解析列规格（列名或列号）
fn parse_column_spec(spec: &str, headers: &[String]) -> Result<usize> {
    // 首先尝试解析为数字
    if let Ok(num) = spec.parse::<usize>() {
        if num == 0 {
            return Err(csv_tool::error::CsvError::Format(
                "列号从1开始".to_string()
            ).into());
        }
        return Ok(num - 1); // 转换为0索引
    }
    
    // 尝试匹配列名
    for (i, header) in headers.iter().enumerate() {
        if header.eq_ignore_ascii_case(spec) {
            return Ok(i);
        }
    }
    
    Err(csv_tool::error::CsvError::Format(
        format!("未找到列 '{}'. 可用的列: {:?}", spec, headers)
    ).into())
}

/// 打印搜索结果
fn print_search_results(
    results: &[csv_tool::csv::SearchResult],
    headers: &[String],
    show_line_numbers: bool,
    highlight: bool,
    max_display: usize,
) {
    let col_count = headers.len().max(
        results.first().map(|r| r.record.fields.len()).unwrap_or(0)
    );
    let max_width = 18;
    
    let separator = "─".repeat(max_width + 2);
    let line_num_width = if show_line_numbers { 8 } else { 0 };
    let line_num_sep = if show_line_numbers { "─".repeat(line_num_width) } else { String::new() };
    
    // 表头
    println!();
    if show_line_numbers {
        print!("┌{}┬", line_num_sep);
    } else {
        print!("┌");
    }
    println!("{}┐", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┬"));
    
    // 列名行
    if show_line_numbers {
        print!("│ {:^6} │", "行号");
    } else {
        print!("│");
    }
    for header in headers.iter().take(col_count) {
        print!(" {:^width$} │", truncate_str(header, max_width), width = max_width);
    }
    for _ in headers.len()..col_count {
        print!(" {:^width$} │", "", width = max_width);
    }
    println!();
    
    // 分隔线
    if show_line_numbers {
        print!("├{}┼", line_num_sep);
    } else {
        print!("├");
    }
    println!("{}┤", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┼"));
    
    // 数据行
    for result in results.iter().take(max_display) {
        if show_line_numbers {
            print!("│ {:>6} │", result.row_number + 1);
        } else {
            print!("│");
        }
        
        for (col_idx, field) in result.record.fields.iter().enumerate().take(col_count) {
            let text = field.as_ref();
            
            // 检查是否需要高亮
            let display_text = if highlight {
                if let Some(match_info) = result.matches.iter().find(|m| m.column == col_idx) {
                    // 高亮匹配文本
                    let highlighted = highlight_matches(text, &match_info.positions);
                    truncate_str_with_ansi(&highlighted, max_width)
                } else {
                    truncate_str(text, max_width)
                }
            } else {
                truncate_str(text, max_width)
            };
            
            print!(" {:width$} │", display_text, width = max_width);
        }
        
        for _ in result.record.fields.len()..col_count {
            print!(" {:width$} │", "", width = max_width);
        }
        println!();
    }
    
    // 底部边框
    if show_line_numbers {
        print!("└{}┴", line_num_sep);
    } else {
        print!("└");
    }
    println!("{}┘", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┴"));
}

/// 创建加载动画
fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// 打印表格
fn print_table(
    headers: &[String], 
    rows: &[csv_tool::csv::CsvRecord], 
    page: usize, 
    total_pages: usize,
    page_size: usize,
) {
    // 计算列宽（根据内容自适应，最大20字符）
    let col_count = headers.len().max(rows.first().map(|r| r.fields.len()).unwrap_or(0));
    let max_width = 18;
    
    let separator = "─".repeat(max_width + 2);
    let full_separator = format!("├{}┤", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┼"));
    
    // 表头
    println!();
    println!("┌{}┐", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┬"));
    
    if !headers.is_empty() {
        print!("│");
        for header in headers.iter().take(col_count) {
            print!(" {:^width$} │", truncate_str(header, max_width), width = max_width);
        }
        // 填充空列
        for _ in headers.len()..col_count {
            print!(" {:^width$} │", "", width = max_width);
    }
    println!();
        println!("{}", full_separator);
    }
    
    // 数据行
    for row in rows {
        print!("│");
        for field in row.fields.iter().take(col_count) {
            print!(" {:width$} │", truncate_str(field.as_ref(), max_width), width = max_width);
        }
        // 填充空列
        for _ in row.fields.len()..col_count {
            print!(" {:width$} │", "", width = max_width);
        }
        println!();
    }
    
    println!("└{}┘", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┴"));
    
    // 分页信息
    let start_row = page * page_size + 1;
    let end_row = start_row + rows.len() - 1;
    println!("📖 第 {}/{} 页 (行 {}-{})", page + 1, total_pages, start_row, end_row);
}

/// 截断字符串
fn truncate_str(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_len {
        let truncated: String = chars[..max_len - 2].iter().collect();
        format!("{}..", truncated)
    } else {
        s.to_string()
    }
}

/// 截断带有ANSI转义序列的字符串
fn truncate_str_with_ansi(s: &str, max_len: usize) -> String {
    // 简化处理：如果字符串包含ANSI代码，直接返回（已经高亮的文本）
    if s.contains('\x1b') {
        // 计算可见字符数
        let visible_len = s.chars().filter(|&c| c != '\x1b').count() 
            - s.matches("\x1b[").count() * 4; // 减去ANSI代码长度估计
        if visible_len > max_len + 10 { // 给高亮代码留余量
            return truncate_str(&s.replace("\x1b[1;33m", "").replace("\x1b[0m", ""), max_len);
        }
        s.to_string()
    } else {
        truncate_str(s, max_len)
    }
}

/// 截断路径显示
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() > max_len {
        format!("...{}", &path[path.len() - max_len + 3..])
    } else {
        path.to_string()
    }
}

/// 导出CSV数据
fn cmd_export(
    args: &Args,
    output: &str,
    format: Option<&str>,
    columns: Option<&str>,
    from: Option<usize>,
    to: Option<usize>,
    search: Option<&str>,
    use_regex: bool,
    pretty: bool,
    no_headers: bool,
) -> Result<()> {
    let start_time = Instant::now();
    let output_path = Path::new(output);
    
    // 确定导出格式
    let export_format = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "json" => ExportFormat::Json,
            "jsonl" | "ndjson" => ExportFormat::JsonLines,
            "csv" => ExportFormat::Csv,
            "tsv" => ExportFormat::Tsv,
            _ => return Err(csv_tool::error::CsvError::Format(
                format!("不支持的格式: {}. 支持的格式: json, jsonl, csv, tsv", fmt)
            ).into()),
        }
    } else {
        // 从文件扩展名推断
        ExportFormat::from_extension(output_path).unwrap_or(ExportFormat::Json)
    };
    
    if !args.quiet {
        println!("\n📤 导出配置:");
        println!("   输出文件: {}", output);
        println!("   导出格式: {}", export_format.name());
    }
    
    let pb = create_spinner("正在打开文件...");
    
    let reader = CsvReader::open_fast(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    pb.finish_and_clear();
    
    let info = reader.info();
    let headers = info.headers.clone();
    
    // 解析列选择
    let export_columns = if let Some(cols_str) = columns {
        let cols: Result<Vec<usize>> = cols_str.split(',')
            .map(|s| parse_column_spec(s.trim(), &headers))
            .collect();
        Some(cols?)
    } else {
        None
    };
    
    // 创建导出选项
    let mut options = ExportOptions::new(export_format)
        .with_pretty(pretty)
        .with_headers(!no_headers)
        .with_delimiter(args.delimiter as u8);
    
    if let Some(cols) = export_columns {
        if !args.quiet {
            println!("   导出列:   {:?}", cols.iter().map(|&i| headers.get(i).cloned().unwrap_or_default()).collect::<Vec<_>>());
        }
        options = options.with_columns(cols);
    }
    
    // 行范围
    if from.is_some() || to.is_some() {
        let start = from.map(|f| f.saturating_sub(1)).unwrap_or(0);
        let end = to.unwrap_or(info.total_rows);
        if !args.quiet {
            println!("   行范围:   {} - {}", start + 1, end);
        }
        options = options.with_row_range(start, end);
    }
    
    // 搜索筛选
    if let Some(pattern) = search {
        if !args.quiet {
            println!("   搜索筛选: \"{}\" {}", pattern, if use_regex { "(正则)" } else { "" });
        }
        let search_pattern = if use_regex {
            SearchPattern::regex(pattern, true)?
        } else {
            SearchPattern::text(pattern, true)
        };
        let search_opts = SearchOptions::new(search_pattern);
        options = options.with_search_filter(search_opts);
    }
    
    // 执行导出
    let pb = create_spinner("正在导出...");
    
    let exporter = Exporter::new(&reader, options);
    let stats = exporter.export_to_file(output)?;
    
    pb.finish_and_clear();
    
    let duration = start_time.elapsed();
    
    println!("\n✅ 导出完成!");
    println!("   导出行数: {} 行", stats.rows_exported);
    println!("   导出列数: {} 列", stats.cols_exported);
    println!("   文件大小: {}", format_size(stats.file_size));
    println!("   输出文件: {}", output);
    println!("   耗时:     {:.2}秒", duration.as_secs_f64());
    
    Ok(())
}

/// 排序子命令
fn cmd_sort(
    args: &Args,
    column: &str,
    order_str: &str,
    data_type_str: &str,
    limit: Option<usize>,
    ignore_case: bool,
    nulls_first: bool,
    show_line_numbers: bool,
    output: Option<&str>,
) -> Result<()> {
    let start_time = Instant::now();
    
    if !args.quiet {
        println!("\n🔢 正在排序数据...");
    }
    
    let pb = create_spinner("正在打开文件...");
    
    let reader = CsvReader::open_fast(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    pb.set_message("正在读取数据...");
    
    let info = reader.info();
    let headers = info.headers.clone();
    
    // 解析列
    let col_idx = parse_column_spec(column, &headers)?;
    
    // 解析排序方向
    let order = SortOrder::from_str(order_str)
        .ok_or_else(|| csv_tool::error::CsvError::Format(
            format!("无效的排序方向: {}，请使用 asc 或 desc", order_str)
        ))?;
    
    // 解析数据类型
    let data_type = DataType::from_str(data_type_str)
        .ok_or_else(|| csv_tool::error::CsvError::Format(
            format!("无效的数据类型: {}，请使用 auto、string 或 number", data_type_str)
        ))?;
    
    if !args.quiet {
        let order_desc = match order {
            SortOrder::Ascending => "升序",
            SortOrder::Descending => "降序",
        };
        let type_desc = match data_type {
            DataType::Auto => "自动",
            DataType::String => "字符串",
            DataType::Number => "数字",
        };
        let col_name = headers.get(col_idx).cloned().unwrap_or_else(|| format!("列{}", col_idx + 1));
        println!("   排序列:   {} ({})", col_name, col_idx + 1);
        println!("   排序方向: {}", order_desc);
        println!("   数据类型: {}", type_desc);
        if let Some(n) = limit {
            println!("   结果限制: {} 行", n);
        }
    }
    
    pb.set_message("正在排序...");
    
    // 创建排序选项
    let sort_key = SortKey::new(col_idx, order, data_type);
    let sort_options = SortOptions::new()
        .add_key(sort_key)
        .with_case_sensitive(!ignore_case)
        .with_nulls_last(!nulls_first);
    
    // 执行排序
    let sorted_records = sort_csv_data(&reader, &sort_options, limit)?;
    
    pb.finish_and_clear();
    
    let duration = start_time.elapsed();
    
    // 输出结果
    if let Some(output_path) = output {
        // 导出到文件
        export_sorted_to_file(&sorted_records, &headers, output_path, args.delimiter as u8)?;
        
        if !args.quiet {
            println!("\n✅ 排序完成!");
            println!("   排序行数: {} 行", sorted_records.len());
            println!("   输出文件: {}", output_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
    } else {
        // 输出到终端
        if !args.quiet {
            println!("\n📊 排序结果 ({} 行，耗时 {:.2}秒):\n", sorted_records.len(), duration.as_secs_f64());
        }
        
        // 准备表头
        let mut display_headers: Vec<String> = Vec::new();
        if show_line_numbers {
            display_headers.push("#".to_string());
        }
        display_headers.extend(headers.iter().cloned());
        
        print_sorted_table(&display_headers, &sorted_records, show_line_numbers);
        
        if !args.quiet {
            println!("\n   共 {} 行", sorted_records.len());
        }
    }
    
    Ok(())
}

/// 打印排序结果表格
fn print_sorted_table(
    headers: &[String],
    records: &[csv_tool::csv::SortedRecord],
    show_line_numbers: bool,
) {
    let col_count = headers.len();
    let max_width = 18;
    
    let separator = "─".repeat(max_width + 2);
    let full_separator = format!("├{}┤", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┼"));
    
    // 表头
    println!();
    println!("┌{}┐", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┬"));
    
    print!("│");
    for header in headers.iter().take(col_count) {
        print!(" {:^width$} │", truncate_str(header, max_width), width = max_width);
    }
    println!();
    
    println!("{}", full_separator);
    
    // 数据行
    for record in records {
        print!("│");
        
        if show_line_numbers {
            print!(" {:>width$} │", record.original_row + 1, width = max_width);
        }
        
        let field_start = if show_line_numbers { 1 } else { 0 };
        for (i, _) in headers.iter().enumerate().skip(field_start) {
            let idx = if show_line_numbers { i - 1 } else { i };
            let value = record.record.fields.get(idx)
                .map(|f| f.as_ref())
                .unwrap_or("");
            print!(" {:^width$} │", truncate_str(value, max_width), width = max_width);
        }
        println!();
    }
    
    // 表底
    println!("└{}┘", (0..col_count).map(|_| separator.clone()).collect::<Vec<_>>().join("┴"));
}

/// 将排序结果导出到文件
fn export_sorted_to_file(
    records: &[csv_tool::csv::SortedRecord],
    headers: &[String],
    output_path: &str,
    delimiter: u8,
) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(output_path)?;
    
    // 写入表头
    writeln!(file, "{}", headers.join(&(delimiter as char).to_string()))?;
    
    // 写入数据行
    for record in records {
        let fields: Vec<String> = record.record.fields
            .iter()
            .map(|f| {
                let s = f.to_string();
                // 如果字段包含分隔符或引号，需要转义
                if s.contains(delimiter as char) || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s
                }
            })
            .collect();
        writeln!(file, "{}", fields.join(&(delimiter as char).to_string()))?;
    }
    
    Ok(())
}

/// 编辑命令
fn cmd_edit(args: &Args, action: &EditAction) -> Result<()> {
    let start_time = Instant::now();
    
    println!("\n✏️  正在编辑文件: {}...", args.file);
    
    let pb = create_spinner("正在打开文件...");
    
    let mut editor = CsvEditor::open(
        &args.file,
        !args.no_headers,
        args.delimiter as u8,
        args.granularity,
    )?;
    
    pb.finish_and_clear();
    
    let headers = editor.headers().to_vec();
    
    match action {
        EditAction::Cell { row, col, value, output } => {
            let col_idx = parse_column_spec(col, &headers)?;
            let row_idx = row.saturating_sub(1); // 转换为0-based
            
            println!("   修改单元格: 行 {}, 列 {} ({})", row, col_idx + 1, 
                headers.get(col_idx).cloned().unwrap_or_default());
            println!("   新值: \"{}\"", value);
            
            editor.edit_cell(row_idx, col_idx, value.clone())?;
            
            let output_path = output.as_deref().unwrap_or(&args.file);
            let options = WriteOptions::new().with_delimiter(args.delimiter as u8);
            
            let pb = create_spinner("正在保存...");
            let stats = if output.is_some() {
                editor.save(output_path, &options)?
            } else {
                editor.save_in_place(&options)?
            };
            pb.finish_and_clear();
            
            let duration = start_time.elapsed();
            println!("\n✅ 编辑完成!");
            println!("   写入行数: {} 行", stats.rows_written);
            println!("   文件大小: {} 字节", stats.bytes_written);
            println!("   输出文件: {}", stats.file_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
        
        EditAction::DeleteRow { rows, output } => {
            let row_nums: Vec<usize> = rows
                .split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();
            
            println!("   删除行: {:?}", row_nums);
            
            for &row in &row_nums {
                editor.delete_row(row.saturating_sub(1))?;
            }
            
            let output_path = output.as_deref().unwrap_or(&args.file);
            let options = WriteOptions::new().with_delimiter(args.delimiter as u8);
            
            let pb = create_spinner("正在保存...");
            let stats = if output.is_some() {
                editor.save(output_path, &options)?
            } else {
                editor.save_in_place(&options)?
            };
            pb.finish_and_clear();
            
            let duration = start_time.elapsed();
            println!("\n✅ 删除完成!");
            println!("   删除行数: {} 行", row_nums.len());
            println!("   剩余行数: {} 行", stats.rows_written);
            println!("   输出文件: {}", stats.file_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
        
        EditAction::AddRow { data, position, output } => {
            let fields: Vec<String> = data.split(',').map(|s| s.trim().to_string()).collect();
            let row = RowData::new(fields);
            
            if let Some(pos) = position {
                println!("   在位置 {} 插入新行", pos);
                editor.insert_row(pos.saturating_sub(1), row)?;
            } else {
                println!("   追加新行到末尾");
                editor.append_row(row)?;
            }
            
            let output_path = output.as_deref().unwrap_or(&args.file);
            let options = WriteOptions::new().with_delimiter(args.delimiter as u8);
            
            let pb = create_spinner("正在保存...");
            let stats = if output.is_some() {
                editor.save(output_path, &options)?
            } else {
                editor.save_in_place(&options)?
            };
            pb.finish_and_clear();
            
            let duration = start_time.elapsed();
            println!("\n✅ 添加完成!");
            println!("   总行数: {} 行", stats.rows_written);
            println!("   输出文件: {}", stats.file_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
        
        EditAction::DeleteCol { cols, output } => {
            let col_specs: Vec<&str> = cols.split(',').map(|s| s.trim()).collect();
            let mut col_indices: Vec<usize> = Vec::new();
            
            for spec in &col_specs {
                let idx = parse_column_spec(spec, &headers)?;
                col_indices.push(idx);
            }
            
            println!("   删除列: {:?}", col_indices.iter()
                .map(|&i| headers.get(i).cloned().unwrap_or_default())
                .collect::<Vec<_>>());
            
            for &col in &col_indices {
                editor.delete_col(col)?;
            }
            
            let output_path = output.as_deref().unwrap_or(&args.file);
            let options = WriteOptions::new().with_delimiter(args.delimiter as u8);
            
            let pb = create_spinner("正在保存...");
            let stats = if output.is_some() {
                editor.save(output_path, &options)?
            } else {
                editor.save_in_place(&options)?
            };
            pb.finish_and_clear();
            
            let duration = start_time.elapsed();
            println!("\n✅ 删除列完成!");
            println!("   删除列数: {} 列", col_indices.len());
            println!("   输出文件: {}", stats.file_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
        
        EditAction::RenameCol { col, name, output } => {
            let col_idx = parse_column_spec(col, &headers)?;
            let old_name = headers.get(col_idx).cloned().unwrap_or_default();
            
            println!("   重命名列: \"{}\" -> \"{}\"", old_name, name);
            
            editor.set_header(col_idx, name.clone())?;
            
            let output_path = output.as_deref().unwrap_or(&args.file);
            let options = WriteOptions::new().with_delimiter(args.delimiter as u8);
            
            let pb = create_spinner("正在保存...");
            let stats = if output.is_some() {
                editor.save(output_path, &options)?
            } else {
                editor.save_in_place(&options)?
            };
            pb.finish_and_clear();
            
            let duration = start_time.elapsed();
            println!("\n✅ 重命名完成!");
            println!("   输出文件: {}", stats.file_path);
            println!("   耗时:     {:.2}秒", duration.as_secs_f64());
        }
    }
    
    Ok(())
}

/// 创建新CSV文件
fn cmd_create(
    output: &str,
    headers_str: &str,
    rows: &[String],
    delimiter: u8,
) -> Result<()> {
    let start_time = Instant::now();
    
    println!("\n📝 正在创建CSV文件: {}...", output);
    
    let headers: Vec<String> = headers_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    println!("   表头: {:?}", headers);
    println!("   数据行数: {}", rows.len());
    
    let options = WriteOptions::new().with_delimiter(delimiter);
    let mut creator = CsvCreator::new(headers.clone()).with_options(options);
    
    for (i, row_str) in rows.iter().enumerate() {
        let fields: Vec<String> = row_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        if fields.len() != headers.len() {
            return Err(csv_tool::error::CsvError::Format(format!(
                "第 {} 行列数 {} 与表头列数 {} 不匹配",
                i + 1, fields.len(), headers.len()
            )));
        }
        
        creator.add_row(RowData::new(fields))?;
    }
    
    let pb = create_spinner("正在保存...");
    let stats = creator.save(output)?;
    pb.finish_and_clear();
    
    let duration = start_time.elapsed();
    
    println!("\n✅ 创建完成!");
    println!("   写入行数: {} 行", stats.rows_written);
    println!("   文件大小: {} 字节", stats.bytes_written);
    println!("   输出文件: {}", stats.file_path);
    println!("   耗时:     {:.2}秒", duration.as_secs_f64());
    
    Ok(())
}
