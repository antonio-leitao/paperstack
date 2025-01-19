// store.js
import { load } from "@tauri-apps/plugin-store";

function generate_unique_id() {
  const timestamp = Date.now();
  const randomNum = Math.floor(Math.random() * 9000) + 1000;
  return `${timestamp}${randomNum}`;
}

let tauriStore = $state(null);
let stacks = $state([]); // Only IDs and names of stacks
let papers = $state([]); // Papers for the currently active stack
let currentStackId = $state(null); // ID of the currently active stack

// Initialize the store (only connect to the Tauri store)
export async function initializeStore() {
  if (!tauriStore) {
    tauriStore = await load("store.json", { autoSave: true });
    stacks = (await tauriStore.get("stacks")) ?? []; // Load only stack metadata
  }
}

export async function resetStore() {
  if (tauriStore) {
    tauriStore.clear();
  }
}

// --- Stack CRUD Operations ---

export async function createStack(name) {
  const stackId = generate_unique_id();
  const newStack = { id: stackId, name };
  stacks = [...stacks, newStack];
  //add metadata
  await tauriStore.set("stacks", stacks);
  //create empty stack
  await tauriStore.set(`${stackId}`, []);
  return stackId;
}

// Get metadata for all stacks (without papers)
export async function getStacks() {
  return stacks;
}

// Get a specific stack (without papers) by ID
export async function getStack(stackId) {
  return stacks.find((stack) => stack.id === stackId);
}

export async function updateStack(stackId, name) {
  stacks = stacks.map((stack) =>
    stack.id === stackId ? { ...stack, name } : stack,
  );
  await tauriStore.set("stacks", stacks);
}

export async function deleteStack(stackId) {
  //dont delete stack while accessing it
  if (currentStackId === stackId) {
    throw new Error("Cannot delete current stack");
  }
  // Delete the stack metadata
  stacks = stacks.filter((stack) => stack.id !== stackId);
  await tauriStore.set("stacks", stacks);

  // Delete the associated papers
  await tauriStore.delete(`${stackId}`);
}

//export function reorderStacks(draggedItemId, targetItemId) {
//  const draggedItemIndex = stacks.findIndex(
//    (stack) => stack.id === draggedItemId,
//  );
//  const targetItemIndex = stacks.findIndex(
//    (stack) => stack.id === targetItemId,
//  );
//
//  if (draggedItemIndex !== -1 && targetItemIndex !== -1) {
//    const [draggedItem] = stacks.splice(draggedItemIndex, 1);
//    stacks.splice(targetItemIndex, 0, draggedItem);
//    stacks = stacks;
//  }
//}

export function reorderStacks(finalOrder) {
  // Create a map of stackId to new index based on finalOrder
  const newOrderMap = new Map();
  finalOrder.forEach((orderItem, index) => {
    newOrderMap.set(orderItem.item, index);
  });

  // Sort the stacks array based on the newOrderMap
  const sortedStacks = [...stacks].sort((a, b) => {
    const indexA = newOrderMap.get(a.id);
    const indexB = newOrderMap.get(b.id);
    return indexA - indexB;
  });
  stacks = sortedStacks;
}

export async function mergeStacks(sourceStackId, targetStackId) {
  const sourceStackPapers = (await tauriStore.get(`${sourceStackId}`)) ?? [];
  const targetStackPapers = (await tauriStore.get(`${targetStackId}`)) ?? [];
  // Merge papers
  const mergedPapers = [...targetStackPapers, ...sourceStackPapers];
  await tauriStore.set(`${targetStackId}`, mergedPapers);
  // Delete the source stack
  await deleteStack(sourceStackId);
}

export async function duplicateStack(stackId) {
  const originalStack = await getStack(stackId);
  if (!originalStack) {
    throw new Error("Stack not found");
  }

  // Create a new stack with a unique ID and modified name
  const newStackId = generate_unique_id();
  const newStackName = `${originalStack.name} copy`;
  const newStack = { id: newStackId, name: newStackName };

  // Duplicate the stack metadata
  stacks = [...stacks, newStack];
  await tauriStore.set("stacks", stacks);

  // Duplicate the papers with new IDs
  const originalPapers = await tauriStore.get(`${stackId}`);
  const duplicatedPapers = originalPapers.map((paper) => ({
    ...paper,
    id: generate_unique_id(),
  }));

  // Store the duplicated papers in the new stack
  await tauriStore.set(`${newStackId}`, duplicatedPapers);

  return newStackId;
}

// --- Paper CRUD Operations ---

// Load papers for a specific stack and set the current stack
export async function loadPapers(stackId) {
  if (!tauriStore) {
    await initializeStore();
  }
  currentStackId = stackId;
  console.log("loading papers from", stackId);
  //TODO throw error if not found
  papers = await tauriStore.get(`${stackId}`);
}

// Create a new paper within a specific stack
export async function createPaper(stackId, paper) {
  const newPaper = { id: generate_unique_id(), ...paper };
  //if it doesnt affect the frontend
  let updatedPapers = [];
  if (currentStackId !== stackId) {
    let other_papers = await tauriStore.get(`${stackId}`);
    updatedPapers = [...other_papers, newPaper];
  } else {
    updatedPapers = [...papers, newPaper];
  }
  await tauriStore.set(`${stackId}`, updatedPapers);
  // Only update the papers state if it's the current stack
  if (currentStackId === stackId) {
    papers = updatedPapers;
  }
}

//// Create a new paper within a specific stack
//export async function createPaper(stackId, paper) {
//  const newPaper = { id: generate_unique_id(), ...paper };
//  const updatedPapers = [...papers, newPaper];
//  await tauriStore.set(`${stackId}`, updatedPapers);
//  // Only update the papers state if it's the current stack
//  if (currentStackId === stackId) {
//    papers = updatedPapers;
//  }
//}

// Update an existing paper within a specific stack
export async function updatePaper(stackId, paperId, updatedPaper) {
  const updatedPapers = papers.map((paper) =>
    paper.id === paperId ? { ...paper, ...updatedPaper } : paper,
  );
  await tauriStore.set(`${stackId}`, updatedPapers);
  // Only update the papers state if it's the current stack
  if (currentStackId === stackId) {
    papers = updatedPapers;
  }
}

// Delete a paper from a specific stack
export async function deletePaper(stackId, paperId) {
  const updatedPapers = papers.filter((paper) => paper.id !== paperId);
  await tauriStore.set(`${stackId}`, updatedPapers);
  // Only update the papers state if it's the current stack
  if (currentStackId === stackId) {
    papers = updatedPapers;
  }
}

// --- Accessors ---
export const Store = {
  initializeStore,
  getStacks,
  createStack,
  getStack,
  updateStack,
  deleteStack,
  mergeStacks,
  loadPapers,
  createPaper,
  updatePaper,
  deletePaper,
  duplicateStack,
  // Expose the reactive states:
  get currentStackId() {
    return currentStackId;
  },
  get stacks() {
    return stacks;
  },
  get papers() {
    return papers;
  },
};
