//! CSV工具性能基准测试
//! 
//! 使用criterion进行性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use csv_tool::csv::CsvReader;
use csv_tool::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn create_large_csv(path: &PathBuf, rows: usize) -> Result<()> {
    let mut file = File::create(path)?;
    
    // 写入表头
    writeln!(file, "id,name,age,city,salary")?;
    
    // 写入数据行
    for i in 1..=rows {
        writeln!(
            file,
            "{},\"Name {}\",{},City {},{}",
            i,
            i,
            20 + i % 50,
            i % 10,
            40000 + (i % 100) * 500
        )?;
    }
    
    Ok(())
}

fn bench_open_file(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_large.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    c.bench_function("open_10k_rows", |b| {
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
}

fn bench_read_first_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_read.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
    
    c.bench_function("read_first_page", |b| {
        b.iter(|| {
            reader.read_page(black_box(0), black_box(20)).unwrap()
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
}

fn bench_read_middle_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_middle.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
    
    c.bench_function("read_middle_page", |b| {
        b.iter(|| {
            reader.read_page(black_box(250), black_box(20)).unwrap()
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
}

fn bench_read_last_page(c: &mut Criterion) {
    let test_file = std::env::temp_dir().join("bench_last.csv");
    create_large_csv(&test_file, 10000).unwrap();
    
    let mut reader = CsvReader::open(&test_file, true, b',', 1000).unwrap();
    let total_pages = reader.total_pages(20);
    
    c.bench_function("read_last_page", |b| {
        b.iter(|| {
            reader.read_page(black_box(total_pages - 1), black_box(20)).unwrap()
        })
    });
    
    // 清理
    std::fs::remove_file(&test_file).ok();
}

criterion_group!(
    benches,
    bench_open_file,
    bench_read_first_page,
    bench_read_middle_page,
    bench_read_last_page
);
criterion_main!(benches);

