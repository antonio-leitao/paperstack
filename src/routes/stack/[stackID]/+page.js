import { Store, loadPapers } from "$lib/state/database.svelte";

export const ssr = false; // Disable SSR for this page

export async function load({ params }) {
  console.log("loading [stackID]: ", params.stackID);
  return {
    response: await loadPapers(params.stackID),
    stackID: params.stackID,
  };
}
