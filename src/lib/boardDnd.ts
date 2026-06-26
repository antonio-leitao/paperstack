// Shared constants and helpers for the project kanban board.
//
// The library palette and every stack column are svelte-dnd-action zones that
// share one `type`, so a card can be dragged between columns and in from the
// library. Library cards carry a prefixed id so they never collide with the
// real project-card ids while a drag is in flight; on drop we strip the prefix
// back to the underlying document id before persisting.

export const BOARD_DND_TYPE = "project-card";
export const LIBRARY_ID_PREFIX = "library:";
export const FLIP_DURATION_MS = 150;

export function libraryCardId(documentId: string): string {
  return `${LIBRARY_ID_PREFIX}${documentId}`;
}

export function realDocumentId(cardId: string): string {
  return cardId.startsWith(LIBRARY_ID_PREFIX)
    ? cardId.slice(LIBRARY_ID_PREFIX.length)
    : cardId;
}
