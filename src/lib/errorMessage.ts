// Normalizes a thrown value (Tauri command rejections arrive as strings, JS
// throws as Error) into a displayable message. Shared by every view that shows
// an error banner.
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
