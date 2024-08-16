use crate::bibfile;
use crate::embedding;
use crate::parser;
use crate::Paper;
use anyhow::{anyhow, Result};
use pdf2image::{RenderOptionsBuilder, DPI, PDF};
use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;

fn save_png(pdf_file: PathBuf, key: String) -> Result<()> {
    let covers_dir = PathBuf::from(tilde("~/.paperstack/covers").to_string());
    // Ensure the parent directory exists;
    fs::create_dir_all(&covers_dir)?;
    // Create or overwrite the file
    let jpeg_path = covers_dir.join(format!("{}.jpeg", key));

    let pdf = PDF::from_file(&pdf_file).unwrap();
    let pages = pdf.render(
        pdf2image::Pages::Single(0),
        RenderOptionsBuilder::default()
            .pdftocairo(true)
            .resolution(DPI::Uniform(50))
            .build()?,
    )?;

    let _ = pages[0].save(jpeg_path);
    Ok(())
}

fn add_paper_to_stack(paper: Paper) -> Result<()> {
    //insert into main bibliography
    let mut bibliography = bibfile::read_bibliography()?;
    bibliography.insert(paper.entry.clone());
    bibfile::save_bibliography(bibliography)?;
    //print sucess
    Ok(())
}

fn add_bibtex(content: &str) -> Result<()> {
    // let content = prompt_message()?;
    let bib = bibfile::read_bibtex(&content)?;
    //get only the first entry
    if let Some(entry) = bib.into_iter().next() {
        let paper = bibfile::parse_entry(entry)
            .map_err(|err| anyhow!("Failed to parse bibliography\n{}", err))?;
        add_paper_to_stack(paper)
    } else {
        Err(anyhow!("Empty bibtex"))
    }
}

#[tokio::main]
pub async fn add_command(arxiv_id: String) -> Result<String> {
    // Run all async functions concurrently and capture their results
    println!("RUST: Adding paper: {}", arxiv_id);
    let (pdf_path, (key, bib, summary)) = tokio::try_join!(
        parser::download_pdf(&arxiv_id),
        parser::arxiv2bib(&arxiv_id),
    )?;
    //add paper to bibliography
    add_bibtex(&bib)?;
    let (pdf_path, _) = tokio::try_join!(
        parser::rename_pdf_file(pdf_path, key.clone()),
        embedding::encode(summary)
    )?;
    save_png(pdf_path, key.clone())?;

    Ok(key)
}
