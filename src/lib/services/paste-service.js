import { extractBibtex } from "./bib-service";
import { fetch } from "@tauri-apps/plugin-http";

/**
 * Handles paste events, extracting images, URLs, BibTeX, or plain text.
 */
export async function readPaste(event) {
  /**
   * Checks if a given URL points to a PDF or an image and returns the appropriate File object or the URL.
   */
  async function identifyUrlType(url) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const contentType = response.headers.get("Content-Type");
      if (!contentType) {
        throw new Error("No content type in response");
      }

      const blob = await response.blob();

      if (contentType === "application/pdf") {
        return {
          type: "PDF",
          content: new File([blob], "pasted.pdf", { type: "application/pdf" }),
        };
      }

      if (contentType.startsWith("image/")) {
        const extension = contentType.split("/")[1];
        return {
          type: "Image",
          content: new File([blob], `pasted.${extension}`, {
            type: contentType,
          }),
        };
      }

      return { type: "URL", content: url };
    } catch (error) {
      console.error("Error identifying URL type:", error);
      return { type: "URL", content: url };
    }
  }

  /**
   * Extracts the image source from an HTML string.
   */
  function extractImageSourceFromHtml(htmlString) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(htmlString, "text/html");
    const imgTag = doc.querySelector("img");
    return imgTag ? imgTag.src : null;
  }

  /**
   * Robustly handles a paste event, processes the clipboard data, and returns the appropriate content.
   */
  async function handlePasteEventRobust(event, callback, options = {}) {
    const {
      allowedImageTypes = ["image/jpeg", "image/png", "image/gif", "image/webp"],
    } = options;

    const clipboardItems = event.clipboardData?.items;
    if (!clipboardItems) {
      console.warn("Clipboard is empty or inaccessible.");
      callback({ type: "Unknown", content: null });
      return;
    }

    for (const item of clipboardItems) {
      if (
        item.kind === "file" &&
        allowedImageTypes.includes(item.type)
      ) {
        const file = item.getAsFile();
        if (file) {
          callback({ type: "Image", content: file });
          return;
        } else {
          console.error("Could not get file from clipboard item.");
        }
      } else if (
        item.kind === "string" &&
        (item.type === "text/plain" || item.type === "text/uri-list")
      ) {
        const text = await new Promise((resolve) => item.getAsString(resolve));

        if (item.type === "text/uri-list") {
            const urls = text.split('\n').filter(isValidUrl);
            if (urls.length > 0) {
                const result = await identifyUrlType(urls[0]); // Considering the first valid URL
                callback(result);
                return;
            }
        } else if (isValidUrl(text)) {
            const result = await identifyUrlType(text);
            callback(result);
            return;
        } else {
            const bibEntry = extractBibtex(text);
            if (bibEntry) {
                callback({ type: "BibTeX", content: bibEntry });
                return;
            } else {
                callback({ type: "Text", content: text });
                return;
            }
        }
      } else if (item.kind === "string" && item.type === "text/html") {
        const htmlString = await new Promise((resolve) =>
          item.getAsString(resolve)
        );
        const imageSrc = extractImageSourceFromHtml(htmlString);
        if (imageSrc) {
          const result = await identifyUrlType(imageSrc);
          callback(result);
          return;
        }
      }
    }

    console.warn("No supported content found in clipboard.");
    callback({ type: "Unknown", content: null });
  }

  /**
   * Simple URL validation helper.
   */
  function isValidUrl(str) {
    try {
      new URL(str);
      return true;
    } catch (_) {
      return false;
    }
  }

  return new Promise((resolve) => {
    handlePasteEventRobust(event, (result) => resolve(result), {
      allowedImageTypes: ["image/jpeg", "image/png", "image/gif", "image/webp"],
    });
  });
}