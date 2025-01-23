import { extractTextFromPDF } from "$lib/services/pdf-service";
import {
  extractBibFromPDF,
  extractSummaryFromPDF,
} from "$lib/services/ai-service";
import { parseBibEntry } from "$lib/services/bib-service";
import { appDataDir, join } from "@tauri-apps/api/path";
import { mkdir, BaseDirectory, remove, writeFile } from "@tauri-apps/plugin-fs";
import { Store, createPaper, updatePaper } from "$lib/state/database.svelte";
import { DialogStore } from "$lib/state/dialog.svelte";

async function confirmUpdate(selected_file, fieldName, newValue = "") {
  let message = `Are you sure you want to replace "${selected_file.bib.title}" ${fieldName}?`;
  if (newValue) {
    message = `Are you sure you want to replace "${selected_file.bib.title}" ${fieldName} with "${newValue}"?`;
  }
  return await DialogStore.confirm(message, "This action cannot be undone.");
}

async function saveFileToAppData(dirName, file, prefix, oldFilePath = null) {
  const appDataDirPath = await appDataDir();
  const timestamp = Date.now();
  const extension = file.name.split(".").pop() || "png";
  const filename = `${prefix}.${extension}`;

  const filePath = await join(appDataDirPath, dirName, filename);

  if (oldFilePath) {
    await remove(oldFilePath);
  }

  const arrayBuffer = await file.arrayBuffer();
  await writeFile(filePath, new Uint8Array(arrayBuffer), {
    baseDir: BaseDirectory.AppData,
  });
  return filePath;
}

async function addOrUpdateContent(stack_id, data, selected_file = null) {
  if (selected_file) {
    await updatePaper(selected_file.id, data);
  } else {
    await createPaper(stack_id, data);
  }
}

export async function addPDFContent(stack_id, pdfFile, selected_file) {
  try {
    if (
      selected_file &&
      !(await confirmUpdate(selected_file, "PDF", pdfFile.name))
    ) {
      return;
    }

    DialogStore.showLoading("Fetching PDF");
    const { text, pages, image } = await extractTextFromPDF(pdfFile);
    DialogStore.updateLoading("Processing PDF");
    const [bibtex, summary] = await Promise.all([
      extractBibFromPDF(text),
      extractSummaryFromPDF(text),
    ]);
    let bib = parseBibEntry(bibtex);
    const prefix = selected_file ? selected_file.bib.id : bib.id;

    const pdfPath = await saveFileToAppData(
      "pdfs",
      pdfFile,
      prefix,
      selected_file?.pdf,
    );

    let imagePath = selected_file?.image || null;
    //add image if there is none
    if (!selected_file || !imagePath) {
      imagePath = await saveFileToAppData("images", image, prefix);
    }
    //update the bib if there is one
    if (selected_file) {
      bib = { ...bib, ...selected_file.bib };
    }
    await addOrUpdateContent(
      stack_id,
      { pages, bibtex, bib, summary, pdf: pdfPath, image: imagePath },
      selected_file,
    );
  } catch (error) {
    console.error("Error processing PDF:", error);
  } finally {
    DialogStore.close();
  }
}

export async function addBibTeXContent(stack_id, bibtex, selected_file) {
  if (selected_file && !(await confirmUpdate(selected_file, "bib entry"))) {
    return;
  }
  const bib = parseBibEntry(bibtex);
  await addOrUpdateContent(stack_id, { bib }, selected_file);
}

export async function addURLContent(stack_id, url, selected_file) {
  if (selected_file) {
    if (selected_file.url && !(await confirmUpdate(selected_file, "URL"))) {
      return;
    }
    await addOrUpdateContent(stack_id, { url }, selected_file);
  }
}

export async function addImageContent(stack_id, imageFile, selected_file) {
  if (selected_file) {
    if (selected_file.image && !(await confirmUpdate(selected_file, "image"))) {
      return;
    }
    const prefix = selected_file.bib.id;
    try {
      const imagePath = await saveFileToAppData(
        "images",
        imageFile,
        prefix,
        selected_file?.image,
      );
      await addOrUpdateContent(stack_id, { image: imagePath }, selected_file);
    } catch (error) {
      console.error("Error saving image:", error);
    }
  }
}
