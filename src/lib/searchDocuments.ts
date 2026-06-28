import type { LibraryDocument } from "./types";

// Fuzzy library search, kept out of the component so it stays presentational and
// this logic is independently testable. Tokenizes the query, scores each
// document across its title/author/filename fields (exact, prefix, substring,
// subsequence and a small typo tolerance), and falls back to recency order when
// the query is empty. A document must match every token to appear.
export function searchDocuments(items: LibraryDocument[], rawQuery: string): LibraryDocument[] {
  const tokens = normalize(rawQuery).split(/\s+/).filter(Boolean);
  if (!tokens.length) return [...items].sort(byRecentlyViewed);

  return items
    .map((document) => ({ document, score: scoreDocument(document, tokens) }))
    .filter((result) => result.score > 0)
    .sort((left, right) => right.score - left.score || byRecentlyViewed(left.document, right.document))
    .map((result) => result.document);
}

function normalize(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

function scoreDocument(document: LibraryDocument, tokens: string[]): number {
  const values = [
    document.referenceTitle ?? "",
    document.title,
    document.referenceAuthors.join(" "),
    document.originalFilename,
  ];
  let total = 0;
  for (const token of tokens) {
    const tokenScore = scoreToken(token, values);
    if (!tokenScore) return 0;
    total += tokenScore;
  }
  return total + Math.min(document.lastViewedAt / 1_000_000_000, 10);
}

function scoreToken(token: string, values: string[]): number {
  let best = 0;
  for (const value of values) {
    const normalized = normalize(value);
    if (!normalized) continue;
    if (normalized === token) best = Math.max(best, 120);
    if (normalized.startsWith(token)) best = Math.max(best, 105);
    const index = normalized.indexOf(token);
    if (index >= 0) best = Math.max(best, 95 - Math.min(index, 30));
    if (isSubsequence(token, normalized)) best = Math.max(best, 55);
    for (const word of normalized.split(/\s+/)) {
      if (word.startsWith(token)) best = Math.max(best, 100);
      const distance = typoDistance(token, word);
      if (distance !== null) best = Math.max(best, 78 - distance * 12);
    }
  }
  return best;
}

function isSubsequence(needle: string, haystack: string): boolean {
  let index = 0;
  for (const character of haystack) {
    if (character === needle[index]) index += 1;
    if (index === needle.length) return true;
  }
  return false;
}

function typoDistance(left: string, right: string): number | null {
  if (left.length < 4 || right.length < 4 || Math.abs(left.length - right.length) > 2) {
    return null;
  }
  let previous = Array.from({ length: right.length + 1 }, (_, index) => index);
  for (let i = 1; i <= left.length; i += 1) {
    const current = [i];
    for (let j = 1; j <= right.length; j += 1) {
      const cost = left[i - 1] === right[j - 1] ? 0 : 1;
      current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + cost);
    }
    previous = current;
  }
  const distance = previous[right.length];
  return distance <= 2 ? distance : null;
}

function byRecentlyViewed(left: LibraryDocument, right: LibraryDocument): number {
  return right.lastViewedAt - left.lastViewedAt;
}
