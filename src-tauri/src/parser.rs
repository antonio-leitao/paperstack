extern crate quick_xml;
use crate::fmt::Clean;
use anyhow::{bail, Result};
use regex::Regex;
use reqwest::get;
use serde::Deserialize;
use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;
use tokio::fs::{rename, File};
use tokio::io::AsyncWriteExt;

const STOP_WORD: [&str; 34] = [
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into", "is", "it",
    "no", "not", "of", "on", "or", "such", "that", "the", "their", "then", "there", "these",
    "they", "this", "to", "was", "will", "and", "with",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Feed {
    entry: Entry,
}

#[derive(Debug, Deserialize)]
struct Entry {
    published: String,
    summary: String,
    title: String,
    author: Vec<Author>,
    link: Vec<Link>,
    category: Vec<Category>,
}

#[derive(Debug, Deserialize)]
struct Author {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Link {
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@title")]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Category {
    #[serde(rename = "@term")]
    term: String,
}

fn get_arxiv_pdf_link(arxiv_id: &str) -> String {
    format!("https://arxiv.org/pdf/{}.pdf", arxiv_id)
}

fn first_non_stop_word(entry: &Entry) -> Option<String> {
    let title_words: Vec<&str> = entry.title.split_whitespace().collect();
    for word in title_words {
        let clean_word = remove_non_alphabetic(word);
        if !STOP_WORD.contains(&clean_word.to_lowercase().as_str()) {
            return Some(clean_word);
        }
    }
    None
}

fn remove_non_alphabetic(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z ]").unwrap();
    re.replace_all(input, "").to_string()
}

fn create_key(entry: &Entry) -> Option<String> {
    // Extract the surname of the first author
    let surname = entry.author.get(0).map(|author| {
        author
            .name
            .split_whitespace()
            .last()
            .unwrap_or(&author.name)
            .to_string()
    });
    // Extract the year from the published date
    let year = String::from(&entry.published[..4]);
    // Extract the first word of the title that is not in the stop words list
    let title_word = first_non_stop_word(entry);
    // Combine the surname, year, and the first non-stop word into a single string
    match (surname, title_word) {
        (Some(surname), Some(word)) => Some(format!(
            "{}{}{}",
            surname.to_lowercase(),
            year,
            word.to_lowercase()
        )),
        (Some(_), None) => None,
        (None, Some(_)) => None,
        (None, None) => None,
    }
}

fn generate_biblatex(entry: &Entry, arxiv_id: &str) -> (String, String) {
    let mut biblatex = String::new();
    //key
    biblatex.push_str("@misc{");
    let key = match create_key(entry) {
        Some(key) => key,
        None => arxiv_id.to_string(),
    };
    biblatex.push_str(&key);
    biblatex.push_str(",\n");
    //authors
    biblatex.push_str("    author = {");
    for (i, author) in entry.author.iter().enumerate() {
        if i > 0 {
            biblatex.push_str(" and ");
        }
        biblatex.push_str(&author.name);
    }
    biblatex.push_str("},\n");
    //title
    biblatex.push_str("    title = {");
    biblatex.push_str(&entry.title.clean());
    biblatex.push_str("},\n");
    //arxiv id
    biblatex.push_str("    eprint = {");
    biblatex.push_str(&arxiv_id);
    biblatex.push_str("},\n");
    biblatex.push_str("    archivePrefix = {arXiv},\n");
    //year
    biblatex.push_str("    year = {");
    biblatex.push_str(&entry.published[..4]); // Extracting the year part from the published date
                                              // PDF LINK
    biblatex.push_str("},\n");
    biblatex.push_str("    url = {");
    //get pdf link, use arxiv id if none exists
    if let Some(link) = entry
        .link
        .iter()
        .find(|link| link.title == Some("pdf".to_string()))
    {
        biblatex.push_str(&link.href);
    } else {
        let pdf_link = get_arxiv_pdf_link(&arxiv_id);
        biblatex.push_str(&pdf_link);
    }

    //CATEGORY
    biblatex.push_str("},\n");
    biblatex.push_str("    primaryClass = {");
    if let Some(cat) = entry.category.first() {
        biblatex.push_str(&cat.term);
    }
    biblatex.push_str("},\n");
    biblatex.push_str("}");

    (biblatex, key)
}

fn generate_embedable(entry: &Entry) -> String {
    let mut embedable = String::new();
    //title
    embedable.push_str(&entry.title.clean());
    //abstract
    embedable.push_str(&entry.summary);
    //authors
    for author in entry.author.iter() {
        embedable.push_str(&author.name);
    }
    embedable
}

fn pdf_path(pdf_name: &str) -> Result<PathBuf> {
    // Expand the tilde to the user's home directory
    let base_dir = tilde("~/.paperstack/pdfs").to_string();
    let mut pdfs_path = PathBuf::from(&base_dir);
    // Make sure the directories exist
    fs::create_dir_all(&pdfs_path)?;
    // Append the PDF file name to the path
    pdfs_path.push(format!("{}.pdf", pdf_name));
    // Return the full path as a PathBuf
    Ok(pdfs_path)
}

pub async fn download_pdf(arxiv_id: &str) -> Result<PathBuf> {
    let pdf_url = get_arxiv_pdf_link(arxiv_id);
    let response = reqwest::get(pdf_url).await?;
    let filename = pdf_path(arxiv_id)?;
    let mut file = File::create(&filename).await?;
    let content = response.bytes().await?;
    file.write_all(&content).await?;
    Ok(filename)
}

pub async fn rename_pdf_file(file_path: PathBuf, new_name: String) -> Result<PathBuf> {
    // Get the parent directory of the file
    let parent_dir = match file_path.parent() {
        Some(dir) => dir,
        None => bail!("Trouble renaming the pdf"),
    };
    // Construct the new file path with the new name
    let new_file_path = parent_dir.join(format!("{}.pdf", new_name));
    // Rename the file
    rename(&file_path, &new_file_path).await?;
    Ok(new_file_path)
}

pub async fn arxiv2bib(arxiv_id: &str) -> Result<(String, String, String)> {
    let url = format!(
        "http://export.arxiv.org/api/query?id_list={}&max_results=1",
        arxiv_id
    );
    let response = get(&url).await?;
    let xml = response.text().await?;
    let feed: Feed = quick_xml::de::from_str(&xml)?;
    let (bibtex, key) = generate_biblatex(&feed.entry, arxiv_id);
    let summary = generate_embedable(&feed.entry);
    // Return the results
    Ok((key, bibtex, summary))
}
