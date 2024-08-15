use crate::bibfile;
use crate::{Paper, ReadOnlyPaper};
use anyhow::Result;

fn remove_entry(papers: Vec<Paper>) -> Vec<ReadOnlyPaper> {
    papers
        .into_iter()
        .map(|paper| ReadOnlyPaper {
            id: paper.id,
            author: paper.author,
            year: paper.year,
            title: paper.title,
            slug: paper.slug,
            url: paper.url,
        })
        .collect()
}

pub fn search(_query: String) -> Result<Vec<ReadOnlyPaper>> {
    println!("Scavenging...");
    let bibliography = bibfile::read_bibliography()?;
    let items = bibfile::parse_bibliography(bibliography);
    Ok(remove_entry(items))
}
