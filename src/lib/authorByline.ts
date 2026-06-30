// Builds the single-line "authors + year" byline shown on a paper card:
//   1 author  -> "Vaswani 2017"
//   2 authors -> "Vaswani, Shazeer 2017"
//   3+ authors -> "Vaswani et al. 2017"
// Surname is taken from the part before a comma ("Last, First") or the last
// whitespace token of a "First Last" display name.

function surname(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return "";
  if (trimmed.includes(",")) return trimmed.split(",")[0].trim();
  const tokens = trimmed.split(/\s+/);
  return tokens[tokens.length - 1];
}

export function authorByline(
  authors: string[],
  year: string | null,
): string {
  const names = authors.map(surname).filter(Boolean);
  let people = "";
  if (names.length === 1) people = names[0];
  else if (names.length === 2) people = `${names[0]}, ${names[1]}`;
  else if (names.length > 2) people = `${names[0]} et al.`;
  return [people, year?.trim()].filter(Boolean).join(" ");
}
