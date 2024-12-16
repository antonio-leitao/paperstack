<script lang="ts">
  import { flip } from "svelte/animate";
  import { quintOut } from "svelte/easing";
  import ProgressStep from "$lib/ProgressStep.svelte";
  import FileItem from "$lib/FileItem.svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import { promptUserConfirmation } from "$lib/state/confirmation.svelte.js";
  import { readPaste } from "$lib/services/paste-service.js";
  import { LoadingState } from "$lib/state/loading.svelte";
  import ContextMenu from "$lib/ContextMenu.svelte";
  import InputButton from "$lib/InputButton.svelte";
  import { FileUp } from "lucide-svelte";
  import {
    addPDFContent,
    addURLContent,
    addBibTeXContent,
    addImageContent,
  } from "$lib/services/content-service.js";
  import { remove } from "@tauri-apps/plugin-fs";
  import { Stack,deleteFile } from "$lib/state/database.svelte.js";
  import GridToggle from "$lib/GridToggle.svelte";
  // State
  let { data } = $props();
  console.log(data.params);
  let dragOverGrid = $state(false);
  let drag_id = $state(null);
  let selected_id = $state(null);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let showContextMenu = $state(false);
  let layout_grid = $state(true);
  // Input Event Handlers
  async function handlePasteEvent(event: ClipboardEvent) {
    const payload = await readPaste(event);
    const selected_paper = Stack.papers.find((paper) => paper.id === selected_id);
    switch (payload.type) {
      case "PDF": {
        LoadingState.start("Fetching PDF");
        await addPDFContent(payload.content, selected_paper);
        break;
      }
      case "URL":
        await addURLContent(payload.content, selected_paper);
        break;
      case "BibTeX":
        await addBibTeXContent(payload.content, selected_paper);
        break;
      case "Image":
        await addImageContent(payload.content, selected_paper);
        break;
    }
  }

  async function handleFileInput(event) {
    const selected_paper = Stack.papers.find((paper) => paper.id === selected_id);
    const inputFiles = event.target.files;
    if (!inputFiles) return;
    if (selected_paper) {
      const newFile = inputFiles[0];
      if (newFile.type.includes("pdf")) {
        await addPDFContent(newFile, selected_paper);
      }
      if (newFile.type.includes("image")) {
        await addImageContent(newFile, selected_paper);
      }
    } else {
      for (const file of inputFiles) {
        if (file.type.includes("pdf")) {
          await addPDFContent(file, null);
        }
      }
    }
  }

  // Drag and Drop Handlers
  async function handleDrop(event) {
    let selected_paper = Stack.papers.find((paper) => paper.id === drag_id);
    event.preventDefault();
    const droppedFiles = event.dataTransfer?.files;
    if (!droppedFiles) return;
    if (selected_paper) {
      const newFile = droppedFiles[0];
      if (newFile.type.includes("pdf")) {
        await addPDFContent(newFile, selected_paper);
      }
      if (newFile.type.includes("image")) {
        await addImageContent(newFile, selected_paper);
      }
    } else if (dragOverGrid) {
      //THIS IS WHY WE CANT MERGE BOTH HANDLERS
      for (const file of droppedFiles) {
        if (file.type.includes("pdf")) {
          await addPDFContent(file, null);
        }
      }
    }
    dragOverGrid = false;
    drag_id = null;
  }

  function handleContextMenu(event, paperId) {
    event.preventDefault();
    event.stopPropagation();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    selected_id = paperId;
    showContextMenu = true;
  }

  function handleClick(event) {
    const clickedInside = event.target.closest(".card");
    const clickedContextMenu = event.target.closest(".context-menu");
    const clickedInput = event.target.closest("input");

    if (!clickedContextMenu) {
      showContextMenu = false;
    }

    if (!clickedInside && !clickedContextMenu && !clickedInput) {
      selected_id = null;
    }
  }

  async function handleDelete() {
    if (selected_id) {
      let selected_paper = Stack.papers.find((paper) => paper.id === selected_id);
      //return on cancel
      if (
        !(await promptUserConfirmation(
          "Confirm Delete",
          `Are you sure you want to delete ${selected_paper.bib.title}?`
        ))
      )
        return;
      if (selected_paper.image) {
        await remove(selected_paper.image);
      }
      if (selected_paper.pdf) {
        await remove(selected_paper.pdf);
      }
      await deleteFile(selected_paper.id);
      selected_id = null;
      showContextMenu = false;
    }
  }

  function handleDragEnter(event: DragEvent, paperId: string) {
    event.preventDefault();
    event.stopPropagation();
    drag_id = paperId;
    selected_id = paperId;
  }

  function handleDragOver(event: DragEvent, paperId: string) {
    event.preventDefault();
    event.stopPropagation();
    drag_id = paperId;
    selected_id = paperId;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    // Only clear drag_id if we're actually leaving the element
    if (!event.currentTarget.contains(event.relatedTarget as Node)) {
      drag_id = null;
      selected_id = null;
    }
  }

  function handleGridDragOver(event: DragEvent) {
    event.preventDefault();
    dragOverGrid = true;
    drag_id = null;
  }

  function handleGridDragLeave(event: DragEvent) {
    event.preventDefault();
    dragOverGrid = false;
  }
</script>

<svelte:window onclick={handleClick} onpaste={handlePasteEvent} />

<ContextMenu
  bind:show={showContextMenu}
  x={contextMenuX}
  y={contextMenuY}
  paper={Stack.papers.find((paper) => paper.id === selected_id)}
  {handleDelete}
/>
<ConfirmDialog />
<ProgressStep />

<div>
  <InputButton {handleFileInput} />
  <GridToggle bind:grid={layout_grid} />
</div>

<div
  class="tray"
  class:grid={layout_grid}
  role="grid"
  tabindex="0"
  ondrop={handleDrop}
  ondragover={handleGridDragOver}
  ondragleave={handleGridDragLeave}
  oncontextmenu={(event) => handleContextMenu(event, null)}
  style="border: 2px dashed {dragOverGrid ? 'blue' : 'gray'};"
>
  {#each Stack.papers as paper (paper.id)}
    <div
      class="card"
      class:grid={layout_grid}
      role="gridcell"
      data-file-id={paper.id}
      onclick={() => {
        selected_id = paper.id;
      }}
      oncontextmenu={(event) => handleContextMenu(event, paper.id)}
      onkeydown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          selected_id = paper.id;
        }
      }}
      tabindex="0"
      animate:flip={{ duration: 250, easing: quintOut }}
    >
      <FileItem
        {...paper}
        compactView={!layout_grid}
        isSelected={selected_id === paper.id}
        isBeingDraggedOver={drag_id === paper.id}
        ondragenter={(e) => handleDragEnter(e, paper.id)}
        ondragover={(e) => handleDragOver(e, paper.id)}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
      />
    </div>
  {/each}

  {#if Stack.papers.length === 0}
    <div style="grid-column: 1 / -1; text-align: center; color: gray;">
      <FileUp size={24} /><br />
      Drag and drop files here
    </div>
  {/if}
</div>

<style>
  .tray {
    user-select: none;
    min-height: 300px;
    gap: 10px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .tray.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(12.5rem, 1fr));
    gap: 10px;
    place-items: center;
  }
  .card.grid {
    display: grid;
    place-items: center;
  }
</style>
