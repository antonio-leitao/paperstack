// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod add;
mod bibfile;
mod embedding;
mod fmt;
mod parser;
mod search;
use biblatex::Entry;
use serde::Serialize;

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
        Ok(_) => Ok(String::from("Paper added to stack")),
        Err(e) => Err(e.to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![filter_papers, add_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
