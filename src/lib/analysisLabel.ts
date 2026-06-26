import type { AnalysisStatus } from "./types";

// Short human label for a card's background-analysis status, or null when the
// document is idle (not queued, not processing).
export function analysisLabel(status: AnalysisStatus | undefined): string | null {
  if (!status) return null;
  switch (status.phase) {
    case "queued":
      return "Queued…";
    case "extracting":
      return "Extracting…";
    case "resolving":
      return status.total ? `Resolving ${Math.max(status.total - status.resolved, 0)}…` : "Resolving…";
    case "error":
      return "Analysis failed";
    default:
      return null;
  }
}
