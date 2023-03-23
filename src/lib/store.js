import { writable, readable } from "svelte/store";
import { parse } from "toml";

//consider: function that sets all stores at start of app

export let ownThumbnails = writable({});
export let ownStacks = writable({});

export let featuredThumbnails = readable({});
export let featuredStacks = readable({});

export async function loadOwnData() {
  const response = await fetch("static/demo_thumbnails.toml");
  const tomlString = await response.text();
  const thumbnail_data = parse(tomlString);
  ownThumbnails.set(thumbnail_data);

  const stack_response = await fetch("static/demo_stacks.toml");
  const stack_String = await stack_response.text();
  const stack_data = parse(stack_String);
  ownStacks.set(stack_data);
}

// async function getFeaturedThumbnails() {
//   const response = await fetch("static/demo_featured_thumbnails.toml");
//   const tomlString = await response.text();
//   const data = parse(tomlString);
//   console.log("publications loaded");
//   return writable(data["papers"]);
// }

// async function getOwnStack() {
//   const response = await fetch("static/demo_stacks.toml");
//   const tomlString = await response.text();
//   const data = parse(tomlString);
//   console.log("publications loaded");
//   return writable(data["papers"]);
// }

// async function getFeaturedStack() {
//   const response = await fetch("static/demo_featured_stacks.toml");
//   const tomlString = await response.text();
//   const data = parse(tomlString);
//   console.log("publications loaded");
//   return writable(data["papers"]);
// }
