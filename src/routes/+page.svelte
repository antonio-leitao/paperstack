<script lang="ts">
  import { flip } from "svelte/animate";
  import { quintOut } from "svelte/easing";
  import ProgressStep from "$lib/ProgressStep.svelte";
  import FileItem from "$lib/FileItem.svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import {promptUserConfirmation} from "$lib/state/confirmation.svelte.js"
  import { readPaste } from "$lib/services/paste-service.js";
  import { LoadingState } from "$lib/state/loading.svelte";
  import ContextMenu from "$lib/ContextMenu.svelte";
  import {
    addPDFContent,
    addURLContent,
    addBibTeXContent,
    addImageContent,
  } from "$lib/services/content-service.js";
  import { remove } from "@tauri-apps/plugin-fs";
  import { deleteFile } from "$lib/state/database.svelte";

  // State
  let { data } = $props();
  let files = $state(data.files);
  let dragOverGrid = $state(false);
  let drag_id = $state(null);
  let selected_id = $state(null);
  let contextMenuX = $state(0);
  let contextMenuY = $state(0);
  let showContextMenu = $state(false);

  // Input Event Handlers
  async function handlePasteEvent(event: ClipboardEvent) {
    const payload = await readPaste(event);
    const selected_file = files.find((file) => file.id === selected_id);
    switch (payload.type) {
      case "PDF": {
        LoadingState.start("Fetching PDF");
        files = await addPDFContent(payload.content, selected_file);
        break;
      }
      case "URL":
        files = await addURLContent(payload.content, selected_file);
        break;
      case "BibTeX":
        files = await addBibTeXContent(payload.content, selected_file);
        break;
      case "Image":
        files = await addImageContent(payload.content, selected_file);
        break;
    }
  }

  async function handleFileInput(event) {
    const selected_file = files.find((file) => file.id === selected_id);
    const inputFiles = event.target.files;
    if (!inputFiles) return;
    if (selected_file) {
      const newFile = inputFiles[0];
      files = await addPDFContent(newFile, selected_file);
    } else {
      for (const file of inputFiles) {
        if (file.type.includes("pdf")) {
          files = await addPDFContent(file, null);
        }
      }
    }
  }

  // Drag and Drop Handlers
  async function handleDrop(event) {
    let selected_file = files.find((file) => file.id === drag_id);
    event.preventDefault();
    const droppedFiles = event.dataTransfer?.files;
    if (!droppedFiles) return;
    if (selected_file) {
      const newFile = droppedFiles[0];
      files = await addPDFContent(newFile, selected_file);
    } else if (dragOverGrid) {
      //THIS IS WHY WE CANT MERGE BOTH HANDLERS
      for (const file of droppedFiles) {
        if (file.type.includes("pdf")) {
          files = await addPDFContent(file, null);
        }
      }
    }
    dragOverGrid = false;
    drag_id = null;
  }

  function handleContextMenu(event, fileId) {
    event.preventDefault();
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    selected_id = fileId;
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
      let selected_file = files.find((file) => file.id === selected_id);
	  //return on cancel
	  if (!await promptUserConfirmation(`Are you sure you want to delete ${selected_file.bib.title}?`, "This action cannot be undone.")) return;
      if (selected_file.image) {
        await remove(selected_file.image);
      }
      if (selected_file.pdf) {
        await remove(selected_file.pdf);
      }
      files = await deleteFile(selected_file.id);
      selected_id = null;
      showContextMenu = false;
    }
  }


  function handleDragEnter(event: DragEvent, fileId: string) {
    event.preventDefault();
    event.stopPropagation();
    drag_id = fileId;
    selected_id = fileId;
  }

  function handleDragOver(event: DragEvent, fileId: string) {
    event.preventDefault();
    event.stopPropagation();
    drag_id = fileId;
    selected_id = fileId;
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

<ContextMenu bind:show={showContextMenu} x={contextMenuX} y={contextMenuY} file={files.find((file) => file.id === selected_id)} {handleDelete}/>
<ConfirmDialog />
<ProgressStep/>

<div>
  <input type="file" accept="application/pdf" onchange={handleFileInput} />
</div>

<div
  class="file-grid"
  role="grid"
  tabindex="0"
  ondrop={handleDrop}
  ondragover={handleGridDragOver}
  ondragleave={handleGridDragLeave}
  style="border: 2px dashed {dragOverGrid ? 'blue' : 'gray'}; 
           min-height: 300px; 
           display: grid; 
           grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); 
           gap: 10px; 
           padding: 20px;"
>
  {#each files as file (file.id)}
    <div
      class="card"
      role="gridcell"
      data-file-id={file.id}
      onclick={() => {
        selected_id = file.id;
      }}
      oncontextmenu={(event) => handleContextMenu(event, file.id)}
      onkeydown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          selected_id = file.id;
        }
      }}
      tabindex="0"
      animate:flip={{ duration: 250, easing: quintOut }}
    >
      <FileItem
        {...file}
        isSelected={selected_id === file.id}
        isBeingDraggedOver={drag_id === file.id}
        ondragenter={(e) => handleDragEnter(e, file.id)}
        ondragover={(e) => handleDragOver(e, file.id)}
        ondragleave={handleDragLeave}
        ondrop={handleDrop}
      />
    </div>
  {/each}

  {#if files.length === 0}
    <div style="grid-column: 1 / -1; text-align: center; color: gray;">
      Drag and drop files here
    </div>
  {/if}
</div>


<style>
  .file-grid {
    user-select: none;
  }

</style>
