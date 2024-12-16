// Content Creation Handlers
import { extractTextFromPDF } from "$lib/services/pdf-service";
import { extractBibFromPDF } from "$lib/services/ai-service";
import { parseBibEntry } from "$lib/services/bib-service";
import { appDataDir, join } from "@tauri-apps/api/path";
import { LoadingState } from "$lib/state/loading.svelte";
import { mkdir, BaseDirectory, remove, writeFile } from "@tauri-apps/plugin-fs";
import { createFile, updateFile } from "$lib/state/database.svelte";
import { promptUserConfirmation } from "$lib/state/confirmation.svelte";

/**
 * Handles confirmation prompt for updating existing file content.
 */
async function confirmUpdate(selected_file, fieldName, newValue = "") {
  let message = `Are you sure you want to replace "${selected_file.bib.title}" ${fieldName}?`;
  if (newValue) {
    message = `Are you sure you want to replace "${selected_file.bib.title}" ${fieldName} with "${newValue}"?`;
  }
  return await promptUserConfirmation(message, "This action cannot be undone.");
}

/**
 * Saves a file to the specified directory in AppData.
 */
async function saveFileToAppData(dirName, file, prefix, oldFilePath = null) {
  await mkdir(dirName, { baseDir: BaseDirectory.AppData, recursive: true });
  const appDataDirPath = await appDataDir();
  const timestamp = Date.now();
  const extension = file.name.split(".").pop() || "png";
  const filename = `${prefix}_${timestamp}.${extension}`;
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

/**
 * Adds or updates content for a file based on the provided data.
 */
async function addOrUpdateContent(data, selected_file = null) {
  if (selected_file) {
    await updateFile(selected_file.id, data);
  } else {
    await createFile(data);
  }
}

export async function addPDFContent(pdfFile, selected_file) {
  try {
    // Confirmation before processing
    if (
      selected_file &&
      !(await confirmUpdate(selected_file, "PDF", pdfFile.name))
    ) {
      return;
    }

    //process the pdf
    LoadingState.start("Extracting PDF");
    const { text, pages, image } = await extractTextFromPDF(pdfFile);
    LoadingState.lap("Asking AI");
    const bibtex = await extractBibFromPDF(text);
    let bib = parseBibEntry(bibtex);
    const prefix = selected_file ? selected_file.bib.id : bib.id;

    //save the pdf file
    const pdfPath = await saveFileToAppData(
      "pdfs",
      pdfFile,
      prefix,
      selected_file?.pdf
    );
    //add image if not present
    let imagePath = selected_file?.image || null;
    if (!selected_file || !imagePath) {
      imagePath = await saveFileToAppData("images", image, prefix);
    }

    //add to bib any missing fields from selected_file
    if (selected_file) {
      bib = { ...bib, ...selected_file.bib };
    }

    //save content to database
    await addOrUpdateContent(
      { pages, bibtex, bib, pdf: pdfPath, image: imagePath },
      selected_file
    );
  } catch (error) {
    console.error("Error processing PDF:", error);
  } finally {
    LoadingState.stop();
  }
}

export async function addBibTeXContent(bibtex, selected_file) {
  // Confirmation before processing
  if (selected_file && !(await confirmUpdate(selected_file, "bib entry"))) {
    return;
  }
  const bib = parseBibEntry(bibtex);
  await addOrUpdateContent({ bib }, selected_file);
}

export async function addURLContent(url, selected_file) {
  // Confirmation before processing
  if (selected_file) {
    if (selected_file.url && !(await confirmUpdate(selected_file, "URL"))) {
      return;
    }
    await addOrUpdateContent({ url }, selected_file);
  }
}

export async function addImageContent(imageFile, selected_file) {
  if (selected_file) {
    // Confirmation before processing
    if (selected_file.image && !(await confirmUpdate(selected_file, "image"))) {
      return;
    }

    const prefix = selected_file.bib.id;
    try {
      const imagePath = await saveFileToAppData(
        "images",
        imageFile,
        prefix,
        selected_file?.image
      );
      await addOrUpdateContent({ image: imagePath }, selected_file);
    } catch (error) {
      console.error("Error saving image:", error);
    }
  }
}
