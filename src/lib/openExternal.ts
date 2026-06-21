import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

export async function openExternal(event: MouseEvent, url: string) {
  if (!isTauri()) return;
  event.preventDefault();
  await openUrl(url);
}
