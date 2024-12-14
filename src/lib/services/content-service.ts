// Content Creation Handlers
import { extractTextFromPDF } from "$lib/services/pdf-service";
import { extractBibFromPDF } from "$lib/services/ai-service";
import { parseBibEntry } from "$lib/services/bib-service";
import { appDataDir, join } from "@tauri-apps/api/path";
import { LoadingState } from "$lib/state/loading.svelte";
import { mkdir, BaseDirectory, remove, writeFile } from "@tauri-apps/plugin-fs";
import { createFile, updateFile, getFiles } from "$lib/state/database.svelte";
import { promptUserConfirmation } from "$lib/state/confirmation.svelte";

interface FileUpdateData {
  pages?: number;
  bibtex?: string;
  bib?: any;
  pdf?: string;
  image?: string;
  url?: string;
}

/**
 * Handles the common logic of confirming user action and updating or creating a file.
 */
async function handleFileAction(
  confirmationMessage: string,
  selectedFile: any,
  updateData: FileUpdateData
) {
  if (
    selectedFile &&
    !(await promptUserConfirmation(
      "Confirm Update",
      confirmationMessage,
    ))
  ) {
    return getFiles();
  }

  if (selectedFile) {
    return updateFile(selectedFile.id, updateData);
  } else {
    return createFile(updateData);
  }
}

/**
 * Saves a file to the specified directory in AppData and returns the file path.
 */
async function saveFileToAppData(
  file: File,
  directory: string,
  prefix: string,
  removePrevious?: string
): Promise<string> {
  await mkdir(directory, { baseDir: BaseDirectory.AppData, recursive: true });
  const appDataDirPath = await appDataDir();
  const timestamp = Date.now();
  const extension = file.name.split(".").pop() || "";
  const filename = `${prefix}_${timestamp}${extension ? `.${extension}` : ""}`;
  const filePath = await join(appDataDirPath, directory, filename);

  if (removePrevious) {
    await remove(removePrevious);
  }

  const arrayBuffer = await file.arrayBuffer();
  await writeFile(filePath, new Uint8Array(arrayBuffer), {
    baseDir: BaseDirectory.AppData,
  });
  return filePath;
}

/**
 * Adds or updates content from a PDF file.
 */
export async function addPDFContent(pdfFile: File, selected_file: any) {
  LoadingState.start("Extracting PDF");
  try {
    const confirmationMessage = selected_file
      ? `Are you sure you want to replace "${selected_file.bib.title}" PDF with "${pdfFile.name}"?`
      : null;

    const { text, pages, image } = await extractTextFromPDF(pdfFile);
    LoadingState.lap("Asking AI");
    const bibtex = await extractBibFromPDF(text);
    let bib = parseBibEntry(bibtex);

    // Determine prefix for filenames
    const prefix = selected_file ? selected_file.bib.id : bib.id;

    // Save PDF file
    const pdfPath = await saveFileToAppData(
      pdfFile,
      "pdfs",
      prefix,
      selected_file?.pdf
    );

    let imagePath = selected_file?.image;
    // Save image if creating a new file or the selected file has no image
    if (!selected_file || !imagePath) {
      imagePath = await saveFileToAppData(image, "images", prefix);
    }

    const updateData: FileUpdateData = {
      pages,
      bibtex,
      bib,
      pdf: pdfPath,
      image: imagePath,
    };

    if (selected_file) {
      updateData.bib = { ...bib, ...selected_file.bib }; // Merge bib data
    }

    return await handleFileAction(confirmationMessage, selected_file, updateData);
  } catch (error) {
    console.error("Error processing PDF:", error);
    return getFiles();
  } finally {
    LoadingState.stop();
  }
}

/**
 * Adds or updates content from a BibTeX string.
 */
export async function addBibTeXContent(bibtex: string, selected_file: any) {
  const bib = parseBibEntry(bibtex);
  const confirmationMessage = selected_file
    ? `Are you sure you want to replace "${selected_file.bib.title}" bib entry?`
    : null;
  return handleFileAction(confirmationMessage, selected_file, { bib });
}

/**
 * Adds or updates content from a URL.
 */
export async function addURLContent(url: string, selected_file: any) {
  const confirmationMessage = selected_file
    ? `Are you sure you want to replace "${selected_file.title}" URL?`
    : null;
  return handleFileAction(confirmationMessage, selected_file, { url });
}

/**
 * Adds or updates content from an image file.
 */
export async function addImageContent(imageFile: File, selected_file: any) {
  if (!selected_file || !selected_file.image) {
    return getFiles();
  }

  const confirmationMessage = `Are you sure you want to replace "${selected_file.title}" image?`;
  try {
    const imagePath = await saveFileToAppData(
      imageFile,
      "images",
      selected_file.bib.id,
      selected_file.image
    );
    return handleFileAction(confirmationMessage, selected_file, {
      image: imagePath,
    });
  } catch (error) {
    console.error("Error saving image:", error);
    return getFiles();
  }
}