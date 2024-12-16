import { load } from "@tauri-apps/plugin-store";

function generate_unique_id() {
  // Get current Unix timestamp in milliseconds
  const timestamp = Date.now();
  // Generate a random number between 1000 and 9999
  const randomNum = Math.floor(Math.random() * 9000) + 1000;
  return parseInt(`${timestamp}${randomNum}`);
}

let store = $state(null);
let papers = $state([]);
let stackID = $state(null);
let stacks = $state([]);
export let Navbar = {
  get stacks() {
    return stacks;
  },
  set stacks(val) {
    stacks = val;
  },
};

export let Stack = {
  get stackID() {
    return stackID;
  },
  set stackID(val) {
    stackID = val;
  },
  get papers() {
    return papers;
  },
  set papers(val) {
    papers = val;
  },
};

export async function initializeStacks() {
  if (!store) throw new Error("Store not initialized");
  Stack.papers = (await store.get("stacks")) ?? [];
}

export async function createStack(name) {
  if (!store) throw new Error("Store not initialized");
  try {
    const stacks = (await store.get("stacks")) ?? [];
    let id = generate_unique_id();
    await store.set("stacks", [...stacks, { name, id }]);
    Navbar.stacks = [...stacks, { name, id }];
    return;
  } catch (error) {
    console.error("Error creating stack in store:", error);
    throw error;
  }
}

export async function updateStack(update_id, newName) {
  if (!store) throw new Error("Store not initialized");
  try {
    const stacks = (await store.get("stacks")) ?? [];
    const updatedStacks = stacks.map((stack) =>
      stack.id === update_id ? { ...stack, name: newName } : stack
    );
    await store.set("stacks", updatedStacks);
    Navbar.stacks = updatedStacks;
    return;
  } catch (error) {
    console.error("Error updating stack in store:", error);
    throw error;
  }
}

export async function deleteStack(delete_id) {
  if (!store) throw new Error("Store not initialized");
  try {
    const stacks = (await store.get("stacks")) ?? [];
    const filteredStacks = stacks.filter((stack) => stack.id !== delete_id);
    await store.set("stacks", filteredStacks);
    Navbar.stacks = filteredStacks;
    //MUST ALSO DELETE FILES IN STACK
    return;
  } catch (error) {
    console.error("Error deleting stack from store:", error);
    throw error;
  }
}

//PAPERS

export async function initializeStore(params) {
  if (!store) {
    store = await load("store.json", { autoSave: true });
  }
  console.log("initializeStore", params);
  Stack.stackID = "files";
  Stack.papers = (await store.get("files")) ?? [];
  return;
}

export async function createFile(file) {
  if (!store) throw new Error("Store not initialized");
  try {
    const files = (await store.get("files")) ?? [];
    let id = generate_unique_id();
    await store.set("files", [...files, { ...file, id }]);
    Stack.papers = [...files, { ...file, id }];
    return;
  } catch (error) {
    console.error("Error creating file in store:", error);
    throw error;
  }
}

export async function updateFile(update_id, updatedFile) {
  if (!store) throw new Error("Store not initialized");
  try {
    const files = (await store.get("files")) ?? [];
    const updatedFiles = files.map((file) =>
      file.id === update_id ? { ...file, ...updatedFile } : file
    );
    await store.set("files", updatedFiles);
    Stack.papers = updatedFiles;
    return;
  } catch (error) {
    console.error("Error updating file in store:", error);
    throw error;
  }
}

export async function deleteFile(fileId) {
  if (!store) throw new Error("Store not initialized");
  try {
    const files = (await store.get("files")) ?? [];
    const filteredFiles = files.filter((file) => file.id !== fileId);
    await store.set("files", filteredFiles);
    Stack.papers = filteredFiles;
    return;
  } catch (error) {
    console.error("Error deleting file from store:", error);
    throw error;
  }
}
