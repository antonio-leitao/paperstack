import { isTauri } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

export async function copyToClipboard(text: string): Promise<void> {
  if (!text.trim()) throw new Error("Nothing to copy");

  if (isTauri()) {
    try {
      await writeText(text);
      return;
    } catch {
      throw new Error("Clipboard unavailable");
    }
  }

  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return;
    } catch {
      // Fall through for webviews where the Clipboard API is present but restricted.
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  document.body.appendChild(textarea);
  textarea.select();
  const copied = document.execCommand("copy");
  textarea.remove();
  if (!copied) throw new Error("Clipboard access was denied");
}
