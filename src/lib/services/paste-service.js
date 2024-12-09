import {extractBibtex} from "./bib-service";
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { fetch } from '@tauri-apps/plugin-http'

function extractImageSourceFromHtml(htmlString) {
  const parser = new DOMParser();
  const doc = parser.parseFromString(htmlString, "text/html");
  const imgTag = doc.querySelector("img");
  return imgTag ? imgTag.src : null; // Fixed to return src, not href
}

//function getTextFromElement(element) {
//  return new Promise((resolve) => {
//    element.getAsString((pastedText) => resolve(pastedText));
//  });
//}

async function getTextFromElement(element) {
    const pastedText = await readText();
    return pastedText;
}

function parseLink(text) {
  const urlPattern = /https?:\/\/[^\s/$.?#].[^\s]*/i;
  const match = text.match(urlPattern);
  return match ? match[0] : null;
}

async function identifyUrlType(url) {
    try {
        const response = await fetch(url, { method: 'HEAD' });
        const contentType = response.headers.get('Content-Type');

        // Check for PDF
        if (contentType === 'application/pdf') {
            return 'PDF';
        }

        // Check for image types
        if (contentType && contentType.startsWith('image/')) {
            return 'Image';
        }

        return "URL";
    } catch (error) {
        console.error('Error checking URL:', error);
        return "URL";
    }
}

export async function readPaste(event) {
  const clipboardData = event.clipboardData || window.clipboardData;
  const items = clipboardData.items;

  for (let item of items) {
    const { kind, type } = item;

    if (type.startsWith("image/")) {
      // Handle pasted image blob
      const blob = item.getAsFile();
      const imageUrl = URL.createObjectURL(blob);
      return { type: "Image", content: imageUrl };
    }

    if (kind === "string" && type === "text/html") {
      // Handle pasted HTML content
      const htmlString = await getTextFromElement(item);
      const imageSrc = extractImageSourceFromHtml(htmlString);
      if (imageSrc) {
        return { type: "Image", content: imageSrc };
      }
    }

    if (kind === "string" && type === "text/plain") {
      // Handle pasted plain text
      const pastedText = await readText();

      // Check if the text is a BibTeX entry
      const bibEntry = extractBibtex(pastedText);
      if (bibEntry) {
        return { type: "BibTeX", content: bibEntry };
      }

      // Check if the text is a URL
      const link = parseLink(pastedText);
      if (link) {
        const urlType = await identifyUrlType(link);
        return { type: urlType, content: link };
      }
      // Return plain text if not BibTeX or URL
      return { type: "Text", content: pastedText };
    }
  }
  // Return unknown if no recognizable content was found
  return { type: "Unknown", content: null };
}
