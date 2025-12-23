// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use csv_tool::csv::CsvReader;
use csv_tool::error::Result as CsvResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct CsvFileInfo {
    file_path: String,
    file_size: u64,
    total_rows: usize,
    total_cols: usize,
    headers: Vec<String>,
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

// 全局存储打开的CSV读取器
static READERS: Mutex<HashMap<String, CsvReader>> = Mutex::new(HashMap::new());

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
    
    let granularity = index_granularity.unwrap_or(1000);

    let mut reader = CsvReader::open(&file_path, has_headers, delimiter_byte, granularity)
        .map_err(|e| format!("打开文件失败: {}", e))?;

    let info = reader.info();
    let file_info = CsvFileInfo {
        file_path: info.file_path.to_string_lossy().to_string(),
        file_size: info.file_size,
        total_rows: info.total_rows,
        total_cols: info.total_cols,
        headers: info.headers.clone(),
    };

    // 存储读取器
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
        .ok_or_else(|| "文件未打开".to_string())?;

    let total_pages = reader.total_pages(page_size);
    let rows = reader
        .read_page(page, page_size)
        .map_err(|e| format!("读取页面失败: {}", e))?;

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
        .ok_or_else(|| "文件未打开".to_string())?;

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
            get_file_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

