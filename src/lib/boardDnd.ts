import type { LibraryDocument, ProjectDocument } from "./types";

// Shared data shape for every zone of BOARD_DND_TYPE. A board entry is either a
// single paper or a whole pile; a library entry is the same shape with a single
// member whose projectDocument is null.
export type BoardMember = {
  document: LibraryDocument;
  projectDocument: ProjectDocument | null;
};

export type BoardEntry = {
  id: string;
  pileId: string | null;
  pileName: string | null;
  members: BoardMember[];
  source: "board" | "library";
};

export type BoardDragMode = "idle" | "reorder" | "merge";

export const BOARD_DND_TYPE = "project-card";
export const LIBRARY_ID_PREFIX = "library:";
export const DOCUMENT_ID_PREFIX = "document:";
export const PILE_ID_PREFIX = "pile:";
// An open pile renders a header row as its own board entry (a member-less entry
// that occupies real column space). It carries its pileId so it acts as the top
// boundary of the pile's shell, and dragging it moves the whole pile.
export const HEADER_ID_PREFIX = "header:";
export const CARD_FLIP_DURATION_MS = 120;
export const COLUMN_FLIP_DURATION_MS = 200;

export function libraryEntryId(documentId: string): string {
  return `${LIBRARY_ID_PREFIX}${documentId}`;
}

export function documentEntryId(documentId: string): string {
  return `${DOCUMENT_ID_PREFIX}${documentId}`;
}

export function pileEntryId(pileId: string): string {
  return `${PILE_ID_PREFIX}${pileId}`;
}

export function headerEntryId(pileId: string): string {
  return `${HEADER_ID_PREFIX}${pileId}`;
}

export function isHeaderEntry(entry: BoardEntry): boolean {
  return entry.id.startsWith(HEADER_ID_PREFIX);
}
