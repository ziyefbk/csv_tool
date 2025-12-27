//! CSV工具性能基准测试
//! 
//! 使用criterion进行性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use csv_tool::csv::CsvReader;
use csv_tool::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn create_large_csv(path: &PathBuf, rows: usize) -> Result<()> {
    let mut file = File::create(path)?;
    
    // 写入表头
    writeln!(file, "id,name,age,city,salary,description")?;
    
    // 写入数据行（每行约 100 字节）
    for i in 1..=rows {
        writeln!(
            file,
            "{},\"Name {}\",{},City {},{},\"This is a sample description for row {} with some padding text\"",
            i,
            i,
            20 + i % 50,
            i % 10,
            40000 + (i % 100) * 500,
            i
        )?;
    }
    
    Ok(())
}

/// 删除索引文件
fn remove_index_file(csv_path: &PathBuf) {
    let idx_path = csv_path.with_extension("csv.idx");
    let _ = std::fs::remove_file(idx_path);
}

fn bench_open_file(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_large.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    c.bench_function("open_10k_rows", |b| {
        // 每次迭代前删除索引文件，确保测试的是首次打开
        b.iter(|| {
            remove_index_file(&test_file);
            CsvReader::open(
                black_box(&test_file),
                true,
                b',',
                1000,
            ).unwrap()
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    remove_index_file(&test_file);
}

/// 对比 open 和 open_fast 的性能差异
fn bench_open_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("open_comparison");
    
    // 测试不同大小的文件
    for rows in [10_000, 100_000, 500_000].iter() {
        let test_file = std::env::temp_dir().join(format!("bench_{}k.csv", rows / 1000));
        create_large_csv(&test_file, *rows).unwrap();
        
        // 普通 open（首次打开，需要构建索引）
        group.bench_with_input(
            BenchmarkId::new("open_standard", format!("{}k", rows / 1000)),
            rows,
            |b, _| {
                b.iter(|| {
                    remove_index_file(&test_file);
                    CsvReader::open(
                        black_box(&test_file),
                        true,
                        b',',
                        1000,
                    ).unwrap()
                })
            },
        );
        
        // 快速 open（使用采样估算）
        group.bench_with_input(
            BenchmarkId::new("open_fast", format!("{}k", rows / 1000)),
            rows,
            |b, _| {
                b.iter(|| {
                    remove_index_file(&test_file);
                    CsvReader::open_fast(
                        black_box(&test_file),
                        true,
                        b',',
                        1000,
                    ).unwrap()
                })
            },
        );
        
        // 清理
        std::fs::remove_file(&test_file).ok();
        remove_index_file(&test_file);
    }
    
    group.finish();
}

/// 测试有缓存索引时的打开速度
fn bench_open_with_cached_index(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_cached.csv");
    create_large_csv(&test_file, 100_000).unwrap();
    
    // 首次打开创建索引
    let _ = CsvReader::open(&test_file, true, b',', 1000).unwrap();
    
    c.bench_function("open_100k_with_cached_index", |b| {
        b.iter(|| {
            CsvReader::open(
                black_box(&test_file),
                true,
                b',',
                1000,
            ).unwrap()
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    remove_index_file(&test_file);
}

fn bench_read_first_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_read.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    c.bench_function("read_first_page", |b| {
        let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
        b.iter(|| {
            let records = reader.read_page(black_box(0), black_box(20)).unwrap();
            black_box(records.len())
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    remove_index_file(&test_file);
}

fn bench_read_middle_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_middle.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    c.bench_function("read_middle_page", |b| {
        let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
        b.iter(|| {
            let records = reader.read_page(black_box(250), black_box(20)).unwrap();
            black_box(records.len())
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    remove_index_file(&test_file);
}

fn bench_read_last_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_last.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    c.bench_function("read_last_page", |b| {
        let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
        let total_pages = reader.total_pages(20);
        b.iter(|| {
            let records = reader.read_page(black_box(total_pages - 1), black_box(20)).unwrap();
            black_box(records.len())
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
    remove_index_file(&test_file);
}

criterion_group!(
    benches,
    bench_open_file,
    bench_open_comparison,
    bench_open_with_cached_index,
    bench_read_first_page,
    bench_read_middle_page,
    bench_read_last_page
);
criterion_main!(benches);

