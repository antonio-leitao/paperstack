import type { Reference } from "./types";

const TRUSTED_RESOLUTION_SOURCES = [
  "crossref-",
  "arxiv-",
  "openalex-",
  "semantic-scholar-",
  "manual-bibtex",
];

export function hasTrustedBibtex(reference: Reference): boolean {
  return (
    reference.resolutionStatus === "resolved" &&
    reference.bibtex.trim().length > 0 &&
    TRUSTED_RESOLUTION_SOURCES.some((source) => reference.resolutionSource?.startsWith(source))
  );
}
