import { GoogleGenerativeAI } from "@google/generative-ai";
import { extractBibtex } from "./bib-service";
import Anthropic from "@anthropic-ai/sdk";

const ANTHROPIC_API_KEY = import.meta.env.VITE_ANTHROPIC_API_KEY;
const GOOGLE_API_KEY = import.meta.env.VITE_GOOGLE_API_KEY;

const anthropic = new Anthropic({
  apiKey: ANTHROPIC_API_KEY,
  dangerouslyAllowBrowser: true,
});
//const anthropic = new Anthropic();

export async function generateAnthropicBibTeX(pdfContent) {
  try {
    // Replace placeholders like {{PDF_CONTENT}} with real values,
    // because the SDK does not support variables.
    const msg = await anthropic.messages.create({
      model: "claude-3-5-sonnet-20241022",
      max_tokens: 1000,
      temperature: 0,
      messages: [
        {
          role: "user",
          content: [
            {
              type: "text",
              text: `You are tasked with creating a complete and correctly formatted BibTeX entry from the content of a research article PDF. Follow these steps carefully:\n\n1. First, you will be presented with the content of a PDF file containing a research article. This content will be provided within <pdf_content> tags:\n\n<pdf_content>\n${pdfContent}\n</pdf_content>\n\n2. Carefully analyze the PDF content to extract the following bibliographic information:\n   - Author(s)\n   - Title\n   - Journal or Conference name (if applicable)\n   - Year of publication\n   - Volume and issue (if applicable)\n   - Page numbers (if applicable)\n   - DOI (if available)\n   - Publisher\n   - Any other relevant information specific to the type of publication\n\n3. Based on the type of publication (e.g., article, inproceedings, book), create a BibTeX entry using the appropriate entry type. Follow these guidelines:\n   - Use a citation key that combines the first author's last name, the year, and a word from the title (e.g., smith2023quantum)\n   - Include all relevant fields for the entry type\n   - Ensure proper formatting of author names (Last, First and Last, First)\n   - Enclose the content of each field in curly braces {}\n   - Use double curly braces {{}} for titles to preserve capitalization\n   - Separate multiple authors with \" and \"\n   - Use standard abbreviations for months if needed (jan, feb, mar, etc.)\n\n4. Provide your final BibTeX entry within <bibtex> tags. Ensure that the entry is complete, correctly formatted, and ready for use in a LaTeX document.\n\n<bibtex>\n[Your BibTeX entry goes here]\n</bibtex>\n\nIf you are unable to extract certain information from the PDF content, use your best judgment to create the most complete BibTeX entry possible with the available information. If critical information is missing, indicate this in a comment within the BibTeX entry.`,
            },
          ],
        },
      ],
    });

    //return result.response.text();
    return msg.content[0].text;
  } catch (error) {
    throw new Error("Failed to generate BibTeX entry");
  }
}

const genAI = new GoogleGenerativeAI(GOOGLE_API_KEY);
const model = genAI.getGenerativeModel({ model: "gemini-2.0-flash-exp" });

export async function extractBibFromPDF(pdfContent) {
  try {
    const prompt = `You are tasked with creating a complete and correctly formatted BibTeX entry from the content of a research article PDF. Follow these steps carefully:

1. First, you will be presented with the content of a PDF file containing a research article. This content will be provided within <pdf_content> tags:

<pdf_content>
${pdfContent}
</pdf_content>

2. Carefully analyze the PDF content to extract the following bibliographic information:
   - Author(s)
   - Title
   - Journal or Conference name (if applicable)
   - Year of publication
   - Volume and issue (if applicable)
   - Page numbers (if applicable)
   - DOI (if available)
   - Publisher
   - Any other relevant information specific to the type of publication

3. Based on the type of publication (e.g., article, inproceedings, book), create a BibTeX entry using the appropriate entry type. Follow these guidelines:
   - Use a citation key that combines the first author's last name, the year, and a word from the title (e.g., smith2023quantum)
   - Include all relevant fields for the entry type
   - Ensure proper formatting of author names (Last, First and Last, First)
   - Enclose the content of each field in curly braces {}
   - Use double curly braces {{}} for titles to preserve capitalization
   - Separate multiple authors with " and "
   - Use standard abbreviations for months if needed (jan, feb, mar, etc.)

4. Provide your final BibTeX entry within <bibtex> tags. Ensure that the entry is complete, correctly formatted, and ready for use in a LaTeX document.

<bibtex>
[Your BibTeX entry goes here]
</bibtex>

If you are unable to extract certain information from the PDF content, use your best judgment to create the most complete BibTeX entry possible with the available information. If critical information is missing, indicate this in a comment within the BibTeX entry.`;

    const result = await model.generateContent(prompt);
    let bibtex = extractBibtex(result.response.text());
    return bibtex;
  } catch (error) {
    throw new Error("Failed to generate BibTeX entry");
  }
}

function extractSummary(inputString) {
  // Sanitize input
  if (typeof inputString !== "string") {
    return null;
  }
  const xmlTagMatch = inputString.match(/<summary>([\s\S]*?)<\/summary>/);
  if (xmlTagMatch) {
    const trimmedEntry = xmlTagMatch[1].trim();
    return trimmedEntry.length > 0 ? trimmedEntry : null;
  }
  return null;
}

export async function extractSummaryFromPDF(pdfContent) {
  try {
    const prompt = `You are an expert in distilling complex scientific research into clear, concise summaries. Your task is to create an extremely brief summary of a scientific paper, focusing exclusively on its main findings and novel contributions.

Here's the full text of the research article you need to summarize:

<article_text>
${pdfContent}
</article_text>

Your goal is to produce a summary that meets the following criteria:
1. Extremely brief (2-3 sentences maximum)
2. Focused solely on the research findings and novel contributions
3. Written in active voice with direct, declarative statements
4. Free from academic jargon
5. Does not mention the paper, its authors, or use phrases like "this study shows"

Follow these steps to create your summary:

1. Carefully read the article, paying special attention to the abstract, introduction, and conclusion.

2. Wrap your work inside <research_breakdown> tags, performing the following analysis:
   a) Extract and quote key sentences from the abstract, introduction, and conclusion that highlight the main findings and contributions.
   b) Clearly state the central research question or objective.
   c) List potential novel contributions, numbering them for clarity.
   d) Identify how this research advances the field compared to previous work.
   e) Determine the most significant finding or contribution.
   f) Note any groundbreaking methodologies, but only if they are central to the research's novelty.
   g) Draft 2-3 alternative summaries based on your analysis.
   h) Evaluate each draft summary against the given criteria, noting strengths and weaknesses.
   i) Revise your best draft to ensure it:
      - Is as concise as possible without losing essential information
      - Focuses on what was discovered, not how it was discovered (unless the method is the main contribution)
      - Clearly states the research's main finding and its significance
      - Uses active voice and direct statements
      - Does not mention the paper, study, or researchers at all

3. Present your final summary in <summary> tags.

Remember, your goal is to provide a clear, ultra-concise overview that captures the essence of the research in a way that a general audience can quickly grasp. Focus on the "what" and "why" of the findings, not the "how" unless the methodology is the primary contribution. Most importantly, do not refer to the paper or study itself; present the findings as standalone facts.`;

    const result = await model.generateContent(prompt);
    let summary = extractSummary(result.response.text());
    return summary;
  } catch (error) {
    throw new Error("Failed to generate BibTeX entry");
  }
}
