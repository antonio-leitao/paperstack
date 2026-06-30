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
  members: BoardMember[];
  source: "board" | "library";
};

export const BOARD_DND_TYPE = "project-card";
export const LIBRARY_ID_PREFIX = "library:";
export const DOCUMENT_ID_PREFIX = "document:";
export const PILE_ID_PREFIX = "pile:";
export const FLIP_DURATION_MS = 150;

export function libraryEntryId(documentId: string): string {
  return `${LIBRARY_ID_PREFIX}${documentId}`;
}

export function documentEntryId(documentId: string): string {
  return `${DOCUMENT_ID_PREFIX}${documentId}`;
}

export function pileEntryId(pileId: string): string {
  return `${PILE_ID_PREFIX}${pileId}`;
}

export function entryDocumentIds(entry: BoardEntry): string[] {
  return entry.members.map((member) => member.document.id);
}
