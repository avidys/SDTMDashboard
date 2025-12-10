// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::fs;
mod xpt_parser;
use xpt_parser::XPTParser;

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct FileData { name: String, content: String }

#[tauri::command]
fn read_csv(path: String) -> Result<FileData, String> {
  let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
  let name = std::path::Path::new(&path)
    .file_name().unwrap_or_default().to_string_lossy().to_string();
  Ok(FileData { name, content })
}

#[tauri::command]
fn read_xpt(path: String) -> Result<FileData, String> {
  let data = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
  
  if data.is_empty() {
    return Err("File is empty".to_string());
  }
  
  // Use xpt_parser to parse the XPT file
  let dataset = XPTParser::parse(&data, None)
    .map_err(|e| format!("Failed to parse XPT file: {}", e))?;
  
  eprintln!("Dataset '{}' has {} variables and {} rows", dataset.title, dataset.variables.len(), dataset.rows.len());
  
  if dataset.variables.is_empty() {
    return Err("Dataset has no variables".to_string());
  }
  
  if dataset.rows.is_empty() {
    return Err("Dataset has no rows".to_string());
  }
  
  // Convert to CSV format
  let mut csv = String::new();
  
  // Header row
  let headers: Vec<String> = dataset.variables.iter()
    .map(|v| v.name.clone())
    .collect();
  eprintln!("Headers: {:?}", headers);
  csv.push_str(&headers.join(","));
  csv.push('\n');
  
  // Data rows
  let mut row_count = 0;
  for row in &dataset.rows {
    if row.values.len() != dataset.variables.len() {
      eprintln!("Warning: Row {} has {} values but expected {}", row_count, row.values.len(), dataset.variables.len());
      continue;
    }
    let values: Vec<String> = row.values.iter()
      .map(|v| {
        // Escape values that contain commas or quotes
        if v.contains(',') || v.contains('"') || v.contains('\n') {
          format!("\"{}\"", v.replace('"', "\"\""))
        } else {
          v.clone()
        }
      })
      .collect();
    csv.push_str(&values.join(","));
    csv.push('\n');
    row_count += 1;
  }
  
  eprintln!("Generated CSV with {} rows, {} bytes", row_count, csv.len());
  
  let name = std::path::Path::new(&path)
    .file_name()
    .unwrap_or_default()
    .to_string_lossy()
    .to_string();
  
  Ok(FileData { name, content: csv })
}


fn main() {
  tauri::Builder::default()
//    .plugin(tauri_plugin_log::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_store::Builder::default().build())
    .invoke_handler(tauri::generate_handler![read_csv, read_xpt, read_text_file])
    .run(tauri::generate_context!())
    .expect("error while running tauri");
}
