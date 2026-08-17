import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

// Most links we open come from metadata providers rather than from the user, so
// the scheme is not ours to trust: hand the system opener only web URLs, never a
// `file:` path or some other handler a bad record could name.
function isWebUrl(url: string): boolean {
  try {
    const { protocol } = new URL(url);
    return protocol === "http:" || protocol === "https:";
  } catch {
    return false;
  }
}

export async function openExternal(event: MouseEvent, url: string) {
  if (!isTauri()) return;
  event.preventDefault();
  if (!isWebUrl(url)) return;
  await openUrl(url);
}
