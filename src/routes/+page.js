import { initializeStore, resetStore } from "$lib/state/database.svelte";

export const ssr = false; // Disable SSR for this page

export async function load() {
  console.log("Loading all stacks at root");
  await initializeStore();
  //console.log("Reseting Store");
  //await resetStore();
}
