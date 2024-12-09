import Cite from "citation-js";

export function parseBibEntry(bibtexString) {
  const cite = new Cite(bibtexString);

  // Convert the parsed bibliography into a JSON structure
  const entries = cite.get({ type: "json" });

  // Extract title, year, and authors
    if (entries.length > 0) {
        const entry = entries[0];
        return {
        bibtex:cite.format('bibtex'),
        title: entry.title || "No title available",
        year: entry.issued ? entry.issued["date-parts"]?.[0]?.[0] || "Unknown year" : "Unknown year",
        author: entry.author
        ? entry.author.map(author => `${author.given || ""} ${author.family || ""}`.trim())
        : [] // Return an empty array if no authors are available
            };
        }
    }
    
    export function extractBibtex(inputString) {
        // Sanitize input
        if (typeof inputString !== 'string') {
            return null;
        }
        const xmlTagMatch = inputString.match(/<bibtex>([\s\S]*?)<\/bibtex>/);
        if (xmlTagMatch) {
            const trimmedEntry = xmlTagMatch[1].trim();
            return trimmedEntry.length > 0 ? trimmedEntry : null;
        }
        try {
            // Try to parse the input string with Citation.js
            const citation = new Cite(inputString);
            // Check if it successfully parsed any BibTeX entries
            const bibtex = citation.format('bibtex');
            // Return the BibTeX if found and valid
            return bibtex.trim() || null;
        } catch (error) {
            // If parsing fails, return null
            return null;
        }
    }




// with a good AI this wouldnt be needed
// export function extractBibtex(inputString) {

//     // Check if input is not a string or is empty
//     if (typeof inputString !== 'string' || inputString.trim() === '') {
//         return null;
//     }
    // Approach 1: Using regex (most robust)
//    const regexMatch = inputString.match(/<bibtex>([\s\S]*?)<\/bibtex>/);
//    if (regexMatch && regexMatch[1]) {
//        return regexMatch[1].trim();
//    }
    // If no match found
//    const bibEntryRegex = new RegExp(
//        `@(${BIBTEX_ENTRY_TYPES.join('|')})\\s*{` + // Entry type and opening brace
       // `([^,]+),` + // Citation key (anything before first comma)
        //`([\\s\\S]*?)` + // Entry content (non-greedy to capture full entry)
        //`\\n\\s*}`, // Closing brace (allowing whitespace and newline)
        //'i' // Case-insensitive flag
    //);
    // Try to find a match
    //const match = inputString.match(bibEntryRegex);
    //if (!match) {
    //    return null;
    //}
    // Detailed parsing of the match
    //const [fullMatch, entryType, citationKey, entryContent] = match;
    // Additional validation and cleanup
    //return fullMatch.trim()
//}