import type { PdfHighlightAnnoObject } from "@embedpdf/models";

export type PageSize = {
  page: number;
  width: number;
  height: number;
};

export type PdfBox = {
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type Reference = {
  id: string;
  sourceId: string;
  sharedId: string | null;
  canonicalId: string | null;
  rawCitation: string | null;
  title: string | null;
  authors: string[];
  year: string | null;
  venue: string | null;
  volume: string | null;
  issue: string | null;
  pages: string | null;
  doi: string | null;
  arxivId: string | null;
  pmid: string | null;
  bibtex: string;
  link: string | null;
  resolutionStatus: "resolved" | "identified" | "ambiguous" | "unresolved" | "error";
  resolutionConfidence: number | null;
  resolutionSource: string | null;
  resolutionError: string | null;
  abstractText: string | null;
  openAccessPdf: string | null;
  bibliographyBoxes: PdfBox[];
  calloutBoxes: PdfBox[];
};

export type AnalysisResult = {
  pages: PageSize[];
  sourceReference: Reference | null;
  references: Reference[];
  enrichmentWarning: string | null;
};

export type AnalysisPhase = "queued" | "extracting" | "resolving" | "done" | "error";

export type AnalysisStatus = {
  documentId: string;
  phase: AnalysisPhase;
  resolved: number;
  total: number;
  error: string | null;
};

export type AnalysisProgress = {
  documentId: string;
  analysis: AnalysisResult;
  resolvingReferenceIds: string[];
};

export type Project = {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
  lastOpenedAt: number;
  documentCount: number;
};

export type ProjectStack = {
  id: string;
  projectId: string;
  name: string;
  position: number;
  createdAt: number;
  updatedAt: number;
};

export type DocumentNote = {
  documentId: string;
  text: string;
  createdAt: number;
  updatedAt: number;
};

export type LibraryDocument = {
  id: string;
  contentHash: string;
  originalFilename: string;
  title: string;
  byteSize: number;
  storedPath: string;
  thumbnailPath: string | null;
  note: DocumentNote | null;
  referenceId: string | null;
  referenceBibtex?: string | null;
  referenceTitle: string | null;
  referenceAuthors: string[];
  referenceYear: string | null;
  createdAt: number;
  updatedAt: number;
  lastViewedAt: number;
};

export type LibraryStatistics = {
  projectCount: number;
  paperCount: number;
  referenceCount: number;
};

export type ProviderSettingStatus = {
  hasOverride: boolean;
  hasEnvironmentValue: boolean;
  environmentVariable: string;
};

export type ProviderSettingsStatus = {
  openalex: ProviderSettingStatus;
  semanticScholar: ProviderSettingStatus;
  crossref: ProviderSettingStatus;
};

export type BibtexPreview = {
  citationKey: string;
  entryType: string;
  title: string;
  authors: string[];
  year: string | null;
  venue: string | null;
  doi: string | null;
};

export type ProjectDocument = {
  projectId: string;
  document: LibraryDocument;
  stack: ProjectStack;
  pileId: string | null;
  pileName: string | null;
  position: number;
  addedAt: number;
  updatedAt: number;
};

export type StoredHighlightAnnotation = Omit<PdfHighlightAnnoObject, "created" | "modified"> & {
  created?: Date | string;
  modified?: Date | string;
};

export type DocumentAnnotation = {
  id: string;
  documentId: string;
  kind: "highlight";
  pageIndex: number;
  color: string;
  opacity: number;
  selectedText: string | null;
  annotation: StoredHighlightAnnotation;
  createdAt: number;
  updatedAt: number;
};

// Broadcast by the backend after a committed library mutation, so every window
// (the organizer and any open viewers) can reconcile from the database.
// `action` is "opened" for a last-viewed bump, which needs no full reload.
export type LibraryChangedEvent = {
  kind: string;
  documentId: string | null;
  action: string;
};
