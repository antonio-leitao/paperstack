import * as pdfjs from "pdfjs-dist";
import * as pdfWorker from "pdfjs-dist/build/pdf.worker.mjs";

pdfjs.GlobalWorkerOptions.workerSrc = import.meta.url + "pdfjs-dist/build/pdf.worker.mjs";

export async function extractTextFromPDF(file) {
    if (!file) return { text: "", pageCount: 0, image: null };
  
    try {
      const arrayBuffer = await file.arrayBuffer();
      const loadingTask = pdfjs.getDocument({ data: arrayBuffer });
      const pdf = await loadingTask.promise;
      const pageCount = pdf.numPages;
  
      // Extract text (similar to your original function)
      const textPromises = [];
      for (let i = 1; i <= pageCount; i++) {
        const page = await pdf.getPage(i);
        const textContent = await page.getTextContent();
        const pageText = textContent.items
          .map((item) => item.str || item.char || "")
          .join("");
        textPromises.push(pageText);
      }
  
      // Get the first page as an image
      const firstPage = await pdf.getPage(1);
      const viewport = firstPage.getViewport({ scale: 1 }); // Adjust scale as needed
      const canvas = document.createElement('canvas');
      const context = canvas.getContext('2d');
      canvas.height = viewport.height;
      canvas.width = viewport.width;
  
      const renderContext = {
        canvasContext: context,
        viewport: viewport,
      };
      await firstPage.render(renderContext).promise;
      // Convert canvas to a JPEG data URL (with optimization)
      const imageDataURL = canvas.toDataURL('image/jpeg', 0.8); // 0.8 quality
      // Convert data URL to a Blob (more efficient than returning the data URL directly)
      const imageBlob = await fetch(imageDataURL).then(res => res.blob());
      // Return text, page count, and the image Blob
      const imageFile = new File([imageBlob], `pasted.jpeg`, {
        type: "image/jpeg",
      });

      return {
        text: (await Promise.all(textPromises)).join("\n"),
        pages: pageCount,
        image: imageFile, // Return the image as a Blob
      };
  
    } catch (error) {
      console.error("Error extracting PDF text and image:", error);
      throw new Error("Failed to extract text and image from PDF");
    }
  }

