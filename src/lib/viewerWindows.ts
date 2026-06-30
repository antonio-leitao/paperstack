import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { LibraryDocument } from "./types";

// Opens a paper in its own window, labelled deterministically by document id so
// re-opening the same paper just focuses the window that already shows it.
// Shared by the organizer and by the viewer (when opening a linked reference).
export async function openViewerWindow(document: LibraryDocument): Promise<void> {
  const label = `viewer:${document.id}`;
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    await existing.setFocus();
    return;
  }
  const viewer = new WebviewWindow(label, {
    url: `/viewer?doc=${encodeURIComponent(document.id)}`,
    title: document.referenceTitle ?? document.title ?? "PDF",
    width: 1000,
    height: 800,
    // PDF zoom is handled by EmbedPDF; webview zoom would scale the whole UI.
    zoomHotkeysEnabled: false,
  });
  void viewer.once("tauri://error", (event) => {
    console.error("Could not open viewer window:", event.payload);
  });
}
