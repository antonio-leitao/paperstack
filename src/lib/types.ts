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
  references: Reference[];
  enrichmentWarning: string | null;
};

export type GrobidService = {
  url: string;
  kind: "local" | "hosted";
};
