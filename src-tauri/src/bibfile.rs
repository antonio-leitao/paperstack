use crate::fmt::Clean;
use crate::Paper;
use anyhow::{anyhow, bail, Context, Result};
use biblatex::{Bibliography, Entry, Person, RetrievalError};
use regex::Regex;
use shellexpand::tilde;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

fn parse_year(entry: &Entry) -> Result<i64, RetrievalError> {
    entry.get_as::<i64>("year")
}

fn parse_title(entry: &Entry) -> Result<String, RetrievalError> {
    entry.get_as::<String>("title")
}
fn parse_author(entry: &Entry) -> Result<(String, String), RetrievalError> {
    entry
        .get_as::<Vec<Person>>("author")
        .map(|authors| format_authors(authors))
}
fn format_authors(authors: Vec<Person>) -> (String, String) {
    let formatted = match authors.len() {
        1 => format!("{} {}", authors[0].given_name, authors[0].name),
        2 => format!("{} and {}", authors[0].name, authors[1].name),
        _ => format!("{} et al.", authors[0].name),
    };
    let formatted_authors = formatted.clean();

    let author_line = authors
        .iter()
        .map(|person| person.name.clone())
        .collect::<Vec<_>>()
        .join(" ");

    (formatted_authors, remove_non_alphabetic(&author_line))
}

fn remove_non_alphabetic(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z ]").unwrap();
    re.replace_all(input, "").to_string()
}

fn parse_url(entry: &Entry) -> Option<String> {
    match entry.get_as::<String>("url") {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

pub fn parse_entry(entry: Entry) -> Result<Paper, RetrievalError> {
    let title = parse_title(&entry)?.replace("\\n", "").replace("\\t", "");
    let (author, author_line) = parse_author(&entry)?;
    let year = parse_year(&entry)?;
    let url = parse_url(&entry);
    let slug = format!("{} {} {}", author_line, year, remove_non_alphabetic(&title));
    Ok(Paper {
        id: entry.key.clone(),
        entry,
        author,
        year,
        title,
        slug,
        url,
    })
}

pub fn parse_bibliography(bibliography: Bibliography) -> Vec<Paper> {
    let mut papers: Vec<Paper> = Vec::new();
    for entry in bibliography.into_iter() {
        match parse_entry(entry) {
            Ok(paper) => papers.push(paper),
            Err(_) => continue,
        }
    }
    papers
}

pub fn read_bibtex(bib_content: &str) -> Result<Bibliography> {
    Bibliography::parse(&bib_content)
        .map_err(|err| anyhow!("Failed to parse bibliography\n{}", err))
}

fn get_bib_path() -> PathBuf {
    PathBuf::from(tilde("~/.paperstack/stack.bib").to_string())
}

pub fn save_bibliography(bibliography: Bibliography) -> Result<()> {
    let bib_path = get_bib_path();

    // Check if the path exists and is a directory
    if bib_path.exists() && bib_path.is_dir() {
        bail!("Error: {:?} is a directory, not a file", bib_path);
    }

    // Ensure the parent directory exists
    if let Some(parent) = bib_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    // Create or overwrite the file
    let mut file = fs::File::create(&bib_path)
        .with_context(|| format!("Failed to create or open file for writing: {:?}", bib_path))?;

    file.write_all(bibliography.to_biblatex_string().as_bytes())
        .with_context(|| "Failed to write bibliography content")?;

    Ok(())
}

pub fn read_bibliography() -> Result<Bibliography> {
    let bib_path = get_bib_path();

    // Check if the path exists and is a directory
    if bib_path.exists() && bib_path.is_dir() {
        bail!("Error: {:?} is a directory, not a file", bib_path);
    }
    // If the file doesn't exist, return an empty bibliography
    if !bib_path.exists() {
        return Ok(Bibliography::new());
    }
    let mut bib_content = String::new();
    let mut file = fs::File::open(&bib_path)
        .with_context(|| format!("Failed to open file for reading: {:?}", bib_path))?;

    file.read_to_string(&mut bib_content)
        .with_context(|| "Failed to read bibliography content")?;

    read_bibtex(&bib_content).with_context(|| "Failed to parse bibliography content")
}
