// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bibfile;
mod fmt;
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
        Err(_) => {
            println!("Error loading papers");
            vec![ReadOnlyPaper::default()]
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![filter_papers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
