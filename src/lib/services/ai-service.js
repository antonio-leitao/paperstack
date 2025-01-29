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
    const prompt = `You are an expert scientific summarizer tasked with creating extremely concise summaries of research papers for other experts in the field. Your goal is to distill the paper's key innovation into 2-3 short sentences, focusing solely on what's novel about the research.

First, carefully read the following scientific paper:

<scientific_paper>
${pdfContent}
</scientific_paper>

After reading the paper, follow these steps to create your summary:

1. Extract key sentences: Quote 3-5 sentences from the paper that best highlight the main innovation.

2. List potential innovations: Identify 2-3 potential key innovations from the paper and rank them in order of importance.

3. Identify the key innovation: What is the single most important novel aspect of this research?

4. Draft a summary that:
   - Focuses exclusively on this key innovation
   - Is clear and direct
   - Does not refer to the paper or its authors
   - Does not explain terms or simplify concepts
   - Does not discuss broader impacts
   - Is very short (aim for 2-3 short sentences)

5. Revise your summary to ensure it is as concise and informative as possible for an expert in the field.

6. Check the word count of your summary to ensure it's between 30-50 words.

Work through these steps inside <analysis> tags. Then, present your final summary within <summary> tags.

Example output structure (note that this is a generic example and should not influence the content of your analysis or summary):

<analysis>
1. Key sentences:
   - [Quote 1]
   - [Quote 2]
   - [Quote 3]

2. Potential innovations:
   a) [Innovation 1]
   b) [Innovation 2]
   c) [Innovation 3]

3. Key innovation identified: [Brief statement of the core novel aspect]

4. Initial draft:
[2-3 sentence summary focusing on the innovation]

5. Revision for conciseness:
[Refined 2-3 sentence summary, eliminating any unnecessary words or concepts]

6. Word count: [Number] words
</analysis>

<summary>
[Final 2-3 sentence summary of the paper's key innovation]
</summary>

Remember, your goal is to pinpoint the core innovation as precisely as possible. Avoid broad statements and focus on the specific, novel contribution of the paper.`;

    const result = await model.generateContent(prompt);
    let summary = extractSummary(result.response.text());
    return summary;
  } catch (error) {
    throw new Error("Failed to generate Summary entry");
  }
}
