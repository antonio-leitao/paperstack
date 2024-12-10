// Content Creation Handlers
import { extractTextFromPDF } from "$lib/services/pdf-service";
import { extractBibFromPDF } from "$lib/services/ai-service";
import { parseBibEntry } from "$lib/services/bib-service";
import { appDataDir, join } from "@tauri-apps/api/path";
import { LoadingState } from "$lib/state/loading.svelte";
import { mkdir, BaseDirectory, remove, writeFile } from "@tauri-apps/plugin-fs";
import { createFile, updateFile } from "$lib/state/database.svelte";

export async function addPDFContent(pdfFile, selected_file) {
  LoadingState.start("Extracting PDF");
  try {
    // Extract text and process as before
    const { text, pages } = await extractTextFromPDF(pdfFile);
    LoadingState.lap("Asking AI");
    const bibtex = await extractBibFromPDF(text);
    let bib = parseBibEntry(bibtex);

    // SAVE IT
    await mkdir("pdfs", { baseDir: BaseDirectory.AppData, recursive: true });
    // Get paths
    const appDataDirPath = await appDataDir();
    const timestamp = Date.now();
    // If updating existing file, use its bib.id, otherwise use a temporary name
    const prefix = selected_file ? selected_file.bib.id : bib.id;
    const filename = `${prefix}_${timestamp}.pdf`;
    const pdfPath = await join(appDataDirPath, "pdfs", filename);

    //remove previous pdf
    if (selected_file && selected_file.pdf) {
      await remove(selected_file.pdf);
    }

    // Write the PDF file
    const arrayBuffer = await pdfFile.arrayBuffer();
    await writeFile(pdfPath, new Uint8Array(arrayBuffer), {
      baseDir: BaseDirectory.AppData,
    });

    //UPDATE BACKEND AND UI
    if (selected_file) {
      bib = { ...bib, ...selected_file.bib }; //add to bib any missing fields from selected_file
      return updateFile(selected_file.id, { pages, bibtex, bib, pdf: pdfPath });
    } else {
      return createFile({ pages, bibtex, bib, pdf: pdfPath });
    }
  } catch (error) {
    console.error("Error processing PDF:", error);
  } finally {
    LoadingState.stop();
  }
}

export async function addBibTeXContent(bibtex, selected_file) {
  const bib = parseBibEntry(bibtex);
  if (selected_file) {
    return updateFile(selected_file.id, { bib });
  } else {
    return createFile({ bib });
  }
}

export async function addURLContent(url, selected_file) {
  if (selected_file) {
    return updateFile(selected_file.id, { url });
  }
}
export async function addImageContent(imageFile, selected_file) {
  if (selected_file) {
    try {
      // Create images directory in AppData if it doesn't exist
      await mkdir("images", {
        baseDir: BaseDirectory.AppData,
        recursive: true,
      });

      // Get paths
      const appDataDirPath = await appDataDir();
      const timestamp = Date.now();
      const extension = imageFile.name.split(".").pop() || "png";
      const filename = `${selected_file.bib.id}_${timestamp}.${extension}`;
      const imagePath = await join(appDataDirPath, "images", filename);

      //remove previous image
      if (selected_file.image) {
        await remove(selected_file.image);
      }
      // Write the image file
      const arrayBuffer = await imageFile.arrayBuffer();
      await writeFile(imagePath, new Uint8Array(arrayBuffer), {
        baseDir: BaseDirectory.AppData,
      });

      return updateFile(selected_file.id, { image: imagePath });
    } catch (error) {
      console.error("Error saving image:", error);
    }
  }
}
