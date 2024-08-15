import { invoke } from "@tauri-apps/api/tauri";
import { info } from "../stores.js";

function getArxivId(link) {
  // Check if the link starts with "https://arxiv.org/" or "http://arxiv.org/"
  if (
    link.startsWith("https://arxiv.org/") ||
    link.startsWith("http://arxiv.org/")
  ) {
    // Split the link by "/"
    const parts = link.split("/");
    // Find the index of the "abs" or "pdf" segment
    const index = parts.findIndex((x) => x === "abs" || x === "pdf");
    if (index !== -1 && index + 1 < parts.length) {
      // Return the element after the "abs" or "pdf" segment as the arXiv ID
      return parts[index + 1];
    }
  }
  return null;
}

function removeCommandText(input) {
  // Regular expression to match '@add' followed by any number of spaces
  const regex = /^@add\s*/;
  // Replace the matched prefix and spaces with an empty string
  return input.replace(regex, "");
}

export function add(input) {
  let link = removeCommandText(input);
  let arxivID = getArxivId(link);

  invoke("add_command", { arxivid: arxivID })
    .then((message) => info.set(message))
    .catch((error) => info.set(error));
}
