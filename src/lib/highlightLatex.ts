import type { DocumentAnnotation } from "./types";

export type HighlightLatexDocument = {
  citationKey: string | null;
  pileId?: string | null;
  pileName?: string | null;
  annotations: DocumentAnnotation[];
};

const LATEX_CHARACTER_ESCAPES: Record<string, string> = {
  "\\": "\\textbackslash{}",
  "{": "\\{",
  "}": "\\}",
  "$": "\\$",
  "&": "\\&",
  "#": "\\#",
  "%": "\\%",
  "_": "\\_",
  "^": "\\textasciicircum{}",
  "~": "\\textasciitilde{}",
};

export function escapeLatex(value: string): string {
  return value.replace(/[\\{}$&#%_^~]/g, (character) => LATEX_CHARACTER_ESCAPES[character]);
}

function annotationText(annotation: DocumentAnnotation): string {
  return (
    annotation.selectedText ??
    annotation.annotation.contents ??
    annotation.annotation.custom?.paperstack?.selectedText ??
    ""
  ).trim();
}

function annotationCreated(annotation: DocumentAnnotation): string | null {
  const created = annotation.annotation.created;
  if (created instanceof Date) return created.toISOString();
  return typeof created === "string" && created ? created : null;
}

function cleanSelectedText(value: string): string {
  return value
    .normalize("NFC")
    .replace(/\r\n?/g, "\n")
    // Repair the most common PDF line-wrap hyphenation without touching
    // compounds whose continuation starts with an uppercase character.
    .replace(/(\p{L})-[\t ]*\n[\t ]*(?=\p{Ll})/gu, "$1")
    .replace(/\s+/g, " ")
    .trim();
}

function compareAnnotations(left: DocumentAnnotation, right: DocumentAnnotation): number {
  if (left.pageIndex !== right.pageIndex) return left.pageIndex - right.pageIndex;

  const leftOrigin = left.annotation.rect?.origin;
  const rightOrigin = right.annotation.rect?.origin;
  const vertical = (leftOrigin?.y ?? 0) - (rightOrigin?.y ?? 0);
  if (vertical !== 0) return vertical;

  const horizontal = (leftOrigin?.x ?? 0) - (rightOrigin?.x ?? 0);
  if (horizontal !== 0) return horizontal;

  if (left.createdAt !== right.createdAt) return left.createdAt - right.createdAt;
  return left.id.localeCompare(right.id);
}

function documentHighlightBlocks(document: HighlightLatexDocument): string[] {
  const blocks: string[] = [];
  const seenSelections = new Set<string>();

  for (const annotation of [...document.annotations].sort(compareAnnotations)) {
    const selectedText = annotationText(annotation);
    if (!selectedText) continue;

    // One selection spanning several PDF pages is persisted as one annotation
    // per page. They share both selected text and the annotation creation time.
    const created = annotationCreated(annotation);
    const selectionIdentity = created ? `${created}\u0000${selectedText}` : annotation.id;
    if (seenSelections.has(selectionIdentity)) continue;
    seenSelections.add(selectionIdentity);

    const text = escapeLatex(cleanSelectedText(selectedText));
    if (!text) continue;
    blocks.push(document.citationKey ? `${text} \\cite{${document.citationKey}}` : text);
  }

  return blocks;
}

export function paperHighlightsAsLatex(document: HighlightLatexDocument): string {
  return documentHighlightBlocks(document).join("\n\n");
}

export function pileHighlightsAsLatex(
  pileName: string | null,
  documents: HighlightLatexDocument[],
): string {
  const highlights = documents.flatMap(documentHighlightBlocks);
  if (!highlights.length) return "";
  return `\\subsection{${escapeLatex(pileName?.trim() || "Untitled pile")}}\n\n${highlights.join("\n\n")}`;
}

export function columnHighlightsAsLatex(
  columnName: string,
  documents: HighlightLatexDocument[],
): string {
  const looseHighlights = documents
    .filter((document) => !document.pileId)
    .flatMap(documentHighlightBlocks);

  const piles = new Map<
    string,
    { name: string | null; documents: HighlightLatexDocument[] }
  >();
  for (const document of documents) {
    if (!document.pileId) continue;
    const pile = piles.get(document.pileId);
    if (pile) {
      pile.documents.push(document);
    } else {
      piles.set(document.pileId, {
        name: document.pileName ?? null,
        documents: [document],
      });
    }
  }

  const body: string[] = [];
  if (looseHighlights.length) body.push(looseHighlights.join("\n\n"));
  for (const pile of piles.values()) {
    const latex = pileHighlightsAsLatex(pile.name, pile.documents);
    if (latex) body.push(latex);
  }

  if (!body.length) return "";
  return `\\section{${escapeLatex(columnName.trim() || "Untitled column")}}\n\n${body.join("\n\n")}`;
}
