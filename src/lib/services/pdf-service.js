import * as pdfjs from "pdfjs-dist";
import * as pdfWorker from "pdfjs-dist/build/pdf.worker.mjs";

pdfjs.GlobalWorkerOptions.workerSrc = import.meta.url + "pdfjs-dist/build/pdf.worker.mjs";

export async function extractTextFromPDF(file) {
    if (!file) return "";
    
    try {
        const arrayBuffer = await file.arrayBuffer();
        const pdf = await pdfjs.getDocument({ data: arrayBuffer }).promise;
        const pageCount = pdf.numPages;

        // Extract text from each page
        const textPromises = [];
        for (let i = 1; i <= pageCount; i++) {
            const page = await pdf.getPage(i);
            const textContent = await page.getTextContent();
            const pageText = textContent.items
                .map((item) => item.str || item.char || "")
                .join("");
            textPromises.push(pageText);
        }

        // Combine all page texts
        return {
            text: (await Promise.all(textPromises)).join("\n"),
            pageCount
        };
    } catch (error) {
        console.error("Error extracting PDF text:", error);
        throw new Error("Failed to extract text from PDF");
    }
} 