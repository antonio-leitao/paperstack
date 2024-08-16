// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod add;
mod bibfile;
mod embedding;
mod fmt;
mod parser;
mod search;
//use base64::encode;
use base64::{engine::general_purpose, Engine as _};
use biblatex::Entry;
use image::io::Reader as ImageReader;
use serde::Serialize;
use shellexpand::tilde;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Clone, Default, Serialize)]
pub struct ReadOnlyPaper {
    pub id: String,
    pub author: String,
    pub year: i64,
    pub title: String,
    pub slug: String,
    pub url: Option<String>,
}

#[derive(Clone)]
pub struct Paper {
    pub id: String,
    pub author: String,
    pub year: i64,
    pub title: String,
    pub slug: String,
    pub url: Option<String>,
    pub entry: Entry,
}

#[tauri::command]
fn load_image(key: String) -> Result<String, String> {
    println!("{}", key);

    let img_name = format!("~/.paperstack/covers/{}.jpeg", key);
    let img_path = PathBuf::from(tilde(&img_name).to_string());

    // Read the image into a dynamic image object
    let img = ImageReader::open(img_path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode png: {}", e))?;
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to write to byte array: {}", e))?;
     // Encode the bytes as a base64 string
    let base64_string = general_purpose::STANDARD.encode(&bytes);
    // Print the base64 string
    Ok(base64_string)
}

#[tauri::command]
fn filter_papers(query: String) -> Vec<ReadOnlyPaper> {
    match search::search(query) {
        Ok(items) => items,
        Err(e) => {
            println!("{}", e);
            vec![ReadOnlyPaper::default()]
        }
    }
}

#[tauri::command]
fn add_command(arxivid: String) -> Result<String, String> {
    match add::add_command(arxivid) {
        Ok(key) => Ok(key),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            filter_papers,
            add_command,
            load_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
