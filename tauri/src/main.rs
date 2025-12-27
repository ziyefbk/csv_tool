// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use csv_tool::csv::CsvReader;
use memchr::memchr;
use memmap2::MmapOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct CsvFileInfo {
    file_path: String,
    file_size: u64,
    total_rows: usize,
    total_cols: usize,
    headers: Vec<String>,
}

/// Quick preview result - returns immediately without building index
#[derive(Debug, Serialize, Deserialize)]
struct QuickPreview {
    headers: Vec<String>,
    rows: Vec<CsvRow>,
    file_size: u64,
    estimated_rows: usize,
    is_complete: bool,  // true if small file, false if only preview
}

#[derive(Debug, Serialize, Deserialize)]
struct CsvRow {
    fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PageData {
    rows: Vec<CsvRow>,
    page: usize,
    total_pages: usize,
    page_size: usize,
}

// Global storage for open CSV readers - using LazyLock for Rust 1.80+
static READERS: LazyLock<Mutex<HashMap<String, CsvReader>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
fn open_csv_file(
    file_path: String,
    has_headers: bool,
    delimiter: Option<String>,
    index_granularity: Option<usize>,
) -> std::result::Result<CsvFileInfo, String> {
    let delimiter_byte = delimiter
        .as_ref()
        .and_then(|d| d.as_bytes().first().copied())
        .unwrap_or(b',');
    
    // Dynamic index granularity based on file size
    // Larger files use coarser index to speed up initial loading
    let file_size = std::fs::metadata(&file_path)
        .map(|m| m.len())
        .unwrap_or(0);
    
    let granularity = index_granularity.unwrap_or_else(|| {
        if file_size > 5_000_000_000 {      // > 5GB
            50_000  // Very large files: index every 50,000 rows
        } else if file_size > 1_000_000_000 {  // > 1GB
            20_000  // Large files: index every 20,000 rows
        } else if file_size > 100_000_000 {    // > 100MB
            5_000   // Medium files: index every 5,000 rows
        } else {
            1_000   // Small files: index every 1,000 rows
        }
    });

    // 使用 open_fast 实现毫秒级响应
    let reader = CsvReader::open_fast(&file_path, has_headers, delimiter_byte, granularity)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let info = reader.info();
    let file_info = CsvFileInfo {
        file_path: info.file_path.to_string_lossy().to_string(),
        file_size: info.file_size,
        total_rows: info.total_rows,
        total_cols: info.total_cols,
        headers: info.headers.clone(),
    };

    // Store the reader
    let mut readers = READERS.lock().unwrap();
    readers.insert(file_path.clone(), reader);

    Ok(file_info)
}

#[tauri::command]
fn read_page(
    file_path: String,
    page: usize,
    page_size: usize,
) -> std::result::Result<PageData, String> {
    let mut readers = READERS.lock().unwrap();
    let reader = readers
        .get_mut(&file_path)
        .ok_or_else(|| "File not opened".to_string())?;

    let total_pages = reader.total_pages(page_size);
    let rows = reader
        .read_page(page, page_size)
        .map_err(|e| format!("Failed to read page: {}", e))?;

    let csv_rows: Vec<CsvRow> = rows
        .into_iter()
        .map(|record| CsvRow {
            fields: record.fields.iter().map(|f| f.to_string()).collect(),
        })
        .collect();

    Ok(PageData {
        rows: csv_rows,
        page,
        total_pages,
        page_size,
    })
}

/// Quick preview - read first N rows without building index
/// This allows instant display of large files while index builds in background
#[tauri::command]
fn quick_preview(
    file_path: String,
    preview_rows: usize,
    delimiter: Option<String>,
) -> std::result::Result<QuickPreview, String> {
    let delimiter_byte = delimiter
        .as_ref()
        .and_then(|d| d.as_bytes().first().copied())
        .unwrap_or(b',');

    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata()
        .map(|m| m.len())
        .unwrap_or(0);
    
    let mmap = unsafe { MmapOptions::new().map(&file) }
        .map_err(|e| format!("Failed to mmap file: {}", e))?;

    // Skip BOM if present
    let start = if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" { 3 } else { 0 };

    // Read headers
    let header_end = memchr(b'\n', &mmap[start..])
        .map(|p| start + p)
        .unwrap_or(mmap.len());
    
    let header_line = &mmap[start..header_end];
    let headers = parse_csv_line(header_line, delimiter_byte);
    
    // Read preview rows
    let mut rows = Vec::with_capacity(preview_rows);
    let mut current_pos = header_end + 1;
    let mut line_count = 0;
    
    while current_pos < mmap.len() && line_count < preview_rows {
        let remaining = &mmap[current_pos..];
        let line_end = memchr(b'\n', remaining)
            .map(|p| current_pos + p)
            .unwrap_or(mmap.len());
        
        if line_end > current_pos {
            let line = &mmap[current_pos..line_end];
            let fields = parse_csv_line(line, delimiter_byte);
            rows.push(CsvRow { fields });
            line_count += 1;
        }
        current_pos = line_end + 1;
    }
    
    // Estimate total rows for large files
    let (estimated_rows, is_complete) = if current_pos >= mmap.len() {
        // We read the entire file
        (line_count, true)
    } else {
        // Estimate based on average row size
        let bytes_read = current_pos - start;
        let avg_row_size = bytes_read as f64 / (line_count + 1) as f64;  // +1 for header
        let estimated = ((mmap.len() - start) as f64 / avg_row_size) as usize;
        (estimated.saturating_sub(1), false)  // -1 to exclude header
    };
    
    Ok(QuickPreview {
        headers,
        rows,
        file_size,
        estimated_rows,
        is_complete,
    })
}

/// Parse a single CSV line into fields
fn parse_csv_line(line: &[u8], delimiter: u8) -> Vec<String> {
    // Strip trailing \r for Windows CRLF
    let line = if !line.is_empty() && line[line.len() - 1] == b'\r' {
        &line[..line.len() - 1]
    } else {
        line
    };
    
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    
    for (i, &byte) in line.iter().enumerate() {
        match byte {
            b'"' => in_quotes = !in_quotes,
            _ if byte == delimiter && !in_quotes => {
                let field = String::from_utf8_lossy(&line[start..i]).to_string();
                fields.push(field.trim_matches('"').to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    
    // Add last field
    if start < line.len() {
        let field = String::from_utf8_lossy(&line[start..]).to_string();
        fields.push(field.trim_matches('"').to_string());
    } else {
        fields.push(String::new());
    }
    
    fields
}

#[tauri::command]
fn close_file(file_path: String) -> std::result::Result<(), String> {
    let mut readers = READERS.lock().unwrap();
    readers.remove(&file_path);
    Ok(())
}

#[tauri::command]
fn get_file_info(file_path: String) -> std::result::Result<CsvFileInfo, String> {
    let readers = READERS.lock().unwrap();
    let reader = readers
        .get(&file_path)
        .ok_or_else(|| "File not opened".to_string())?;

    let info = reader.info();
    Ok(CsvFileInfo {
        file_path: info.file_path.to_string_lossy().to_string(),
        file_size: info.file_size,
        total_rows: info.total_rows,
        total_cols: info.total_cols,
        headers: info.headers.clone(),
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_csv_file,
            read_page,
            close_file,
            get_file_info,
            quick_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
