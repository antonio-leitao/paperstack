import { load } from "@tauri-apps/plugin-store";
import { mkdir, BaseDirectory, remove } from "@tauri-apps/plugin-fs";
import { appDataDir, join } from "@tauri-apps/api/path";

// Constants
const UNSORTED_STACK_ID = "unsorted";
const ALL_STACK_ID = "all";

function generate_unique_id() {
  const timestamp = Date.now();
  const randomNum = Math.floor(Math.random() * 9000) + 1000;
  return `${timestamp}${randomNum}`;
}

let tauriStore = $state(null);
let stacks = $state([]); // Array of Stack objects (excluding "unsorted")
let papers = $state([]); // Array of all Paper objects
let unsortedPapers = $state([]); // Array of IDs of unsorted papers
let currentStackId = $state(null); // ID of the currently active stack
let currentPapers = $state([]); // Papers of the current stack

// Initialize the store
export async function initializeStore() {
  if (!tauriStore) {
    tauriStore = await load("store.json", { autoSave: true });
    // Ensure base directories exist
    await mkdir("pdfs", { baseDir: BaseDirectory.AppData, recursive: true });
    await mkdir("images", { baseDir: BaseDirectory.AppData, recursive: true });
    // Load data
    stacks = (await tauriStore.get("stacks")) ?? [];
    papers = (await tauriStore.get("papers")) ?? [];
    unsortedPapers = (await tauriStore.get("unsorted")) ?? [];
  }
}

export async function resetStore() {
  if (tauriStore) {
    await tauriStore.clear();
    stacks = [];
    papers = [];
    unsortedPapers = [];
    currentStackId = null;
    currentPapers = [];
  }
}

// --- Special Stack Accessors ---

export function getAllStack() {
  return {
    id: ALL_STACK_ID,
    name: "All",
    papers: papers.map((p) => p.id), // All papers
  };
}

export function getUnsortedStack() {
  return {
    id: UNSORTED_STACK_ID,
    name: "Unsorted",
    papers: unsortedPapers, // Unsorted paper IDs
  };
}

// --- Stack CRUD Operations ---

export async function createStack(name) {
  const stackId = generate_unique_id();
  const newStack = { id: stackId, name, papers: [] };
  stacks = [...stacks, newStack];
  await tauriStore.set("stacks", stacks);
  return stackId;
}

export async function getStacks() {
  // Exclude "unsorted" from the returned stacks
  return stacks;
}

export async function getStack(stackId) {
  if (stackId === ALL_STACK_ID) {
    return getAllStack();
  }
  if (stackId === UNSORTED_STACK_ID) {
    return getUnsortedStack();
  }
  return stacks.find((stack) => stack.id === stackId);
}

export async function updateStack(stackId, name) {
  stacks = stacks.map((stack) =>
    stack.id === stackId ? { ...stack, name } : stack,
  );
  await tauriStore.set("stacks", stacks);
}

async function deleteStack(stackId) {
  // Move papers to unsorted before deleting the stack
  const stackToDelete = stacks.find((stack) => stack.id === stackId);
  if (stackToDelete) {
    unsortedPapers = [...new Set([...unsortedPapers, ...stackToDelete.papers])];
    await tauriStore.set("unsorted", unsortedPapers);
  }

  // If deleting current stack, switch to "all" view
  if (currentStackId === stackId) {
    currentStackId = ALL_STACK_ID;
  }

  stacks = stacks.filter((stack) => stack.id !== stackId);
  await tauriStore.set("stacks", stacks);
  
  // Update current papers if needed
  updateCurrentPapers();
}

export async function mergeStacks(sourceStackId, targetStackId) {
  const sourceStack = stacks.find((s) => s.id === sourceStackId);
  const targetStack = stacks.find((s) => s.id === targetStackId);

  if (!sourceStack || !targetStack) {
    throw new Error("Stack not found");
  }

  const mergedPaperIds = [
    ...new Set([...targetStack.papers, ...sourceStack.papers]),
  ];

  stacks = stacks.map((stack) =>
    stack.id === targetStackId ? { ...stack, papers: mergedPaperIds } : stack,
  );

  stacks = stacks.filter((stack) => stack.id !== sourceStackId);

  await tauriStore.set("stacks", stacks);
  if (currentStackId === targetStackId) {
    updateCurrentPapers();
  }
}

export async function duplicateStack(stackId) {
  const originalStack = await getStack(stackId);
  if (!originalStack) {
    throw new Error("Stack not found");
  }

  const newStackId = generate_unique_id();
  const newStackName = `${originalStack.name} copy`;
  const newStack = {
    id: newStackId,
    name: newStackName,
    papers: [...originalStack.papers],
  };

  stacks = [...stacks, newStack];
  await tauriStore.set("stacks", stacks);

  return newStackId;
}

export function reorderStacks(finalOrder) {
  const newOrderMap = new Map();
  finalOrder.forEach((orderItem, index) => {
    newOrderMap.set(orderItem.item, index);
  });

  const sortedStacks = [...stacks].sort((a, b) => {
    const indexA = newOrderMap.get(a.id);
    const indexB = newOrderMap.get(b.id);
    return indexA - indexB;
  });
  stacks = sortedStacks;
}

// Updates currentPapers based on current stack's paper IDs
function updateCurrentPapers() {
  if (currentStackId === ALL_STACK_ID) {
    currentPapers = [...papers].sort(
      (a, b) => (b.timestamp || 0) - (a.timestamp || 0),
    );
  } else if (currentStackId === UNSORTED_STACK_ID) {
    currentPapers = papers
      .filter((paper) => unsortedPapers.includes(paper.id))
      .sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0));
  } else {
    const currentStack = stacks.find((s) => s.id === currentStackId);
    if (currentStack) {
      currentPapers = papers
        .filter((paper) => currentStack.papers.includes(paper.id))
        .sort((a, b) => (b.timestamp || 0) - (a.timestamp || 0));
    } else {
      currentPapers = [];
    }
  }
}

// --- Paper CRUD Operations ---

export async function touchPaper(paperId) {
  papers = papers.map((paper) =>
    paper.id === paperId ? { ...paper, timestamp: Date.now() } : paper,
  );
  await tauriStore.set("papers", papers);
  if (currentStackId) {
    updateCurrentPapers();
  }
}

export async function loadPapers(stackId) {
  if (!tauriStore) {
    await initializeStore();
  }
  currentStackId = stackId;
  updateCurrentPapers();
}

export async function findPotentialDuplicate(newPaper) {
  return papers.find((paper) => {
    // Check for matching title if available
    if (newPaper.bib?.title && paper.bib?.title) {
      if (paper.bib.title.toLowerCase() === newPaper.bib.title.toLowerCase()) {
        return paper;
      }
    }
    // Check for matching DOI if available
    if (newPaper.bib?.doi && paper.bib?.doi) {
      if (paper.bib.doi === paper.bib.doi) {
        return paper;
      }
    }
    return null;
  });
}

export async function createPaper(stackId, paper) {
  const newPaper = {
    id: generate_unique_id(),
    timestamp: Date.now(),
    ...paper,
  };

  papers = [...papers, newPaper];
  await tauriStore.set("papers", papers);

  if (stackId && stackId !== UNSORTED_STACK_ID) {
    stacks = stacks.map((stack) =>
      stack.id === stackId
        ? { ...stack, papers: [...stack.papers, newPaper.id] }
        : stack,
    );
    await tauriStore.set("stacks", stacks);
  } else {
    // If no stack ID or unsorted, add to unsortedPapers
    unsortedPapers = [...unsortedPapers, newPaper.id];
    await tauriStore.set("unsorted", unsortedPapers);
  }

  if (currentStackId) {
    updateCurrentPapers();
  }

  return newPaper.id;
}
export async function movePaper(stackId, paperId) {
  if (stackId && stackId !== UNSORTED_STACK_ID) {
    stacks = stacks.map((stack) =>
      stack.id === stackId
        ? { ...stack, papers: [...stack.papers, paperId] }
        : stack,
    );
    await tauriStore.set("stacks", stacks);
  }
  if (currentStackId) {
    updateCurrentPapers();
  }
}

export async function updatePaper(paperId, updatedPaper) {
  papers = papers.map((paper) =>
    paper.id === paperId
      ? { ...paper, ...updatedPaper, timestamp: Date.now() }
      : paper,
  );

  await tauriStore.set("papers", papers);
  if (currentStackId) {
    updateCurrentPapers();
  }
}

export async function removePaperFromStack(paperId, stackId) {
  // Move paper to unsorted
  unsortedPapers = [...new Set([...unsortedPapers, paperId])];
  await tauriStore.set("unsorted", unsortedPapers);

  // Remove paper from specified stack
  stacks = stacks.map((stack) => ({
    ...stack,
    papers:
      stack.id === stackId
        ? stack.papers.filter((id) => id !== paperId)
        : stack.papers,
  }));
  await tauriStore.set("stacks", stacks);

  if (currentStackId) {
    updateCurrentPapers();
  }
}

export async function deletePaper(paperId) {
  const paperToDelete = papers.find((paper) => paper.id === paperId);
  if (paperToDelete) {
    // Delete associated files with error handling
    try {
      if (paperToDelete.pdf) {
        await remove(paperToDelete.pdf);
      }
    } catch (error) {
      console.warn(`Failed to delete PDF for paper ${paperId}:`, error);
    }

    try {
      if (paperToDelete.image) {
        await remove(paperToDelete.image);
      }
    } catch (error) {
      console.warn(`Failed to delete image for paper ${paperId}:`, error);
    }
  }

  // Rest of the function remains the same
  stacks = stacks.map((stack) => ({
    ...stack,
    papers: stack.papers.filter((id) => id !== paperId),
  }));

  papers = papers.filter((paper) => paper.id !== paperId);
  unsortedPapers = unsortedPapers.filter((id) => id !== paperId);

  await tauriStore.set("stacks", stacks);
  await tauriStore.set("papers", papers);
  await tauriStore.set("unsorted", unsortedPapers);

  if (currentStackId) {
    updateCurrentPapers();
  }
}

export const Store = {
  initializeStore,
  reorderStacks,
  duplicateStack,
  getStacks,
  createStack,
  getStack,
  updateStack,
  deleteStack,
  mergeStacks,
  loadPapers,
  findPotentialDuplicate,
  createPaper,
  updatePaper,
  movePaper,
  deletePaper,
  removePaperFromStack,
  touchPaper,
  getAllStack,
  getUnsortedStack,
  get currentStackId() {
    return currentStackId;
  },
  get stacks() {
    return stacks;
  },
  get papers() {
    return currentPapers;
  },
};
