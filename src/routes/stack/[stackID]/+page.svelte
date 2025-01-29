<script lang="ts">
    import { flip } from "svelte/animate";
    import { quintOut } from "svelte/easing";
    import Dialog from "$lib/components/dialog/Dialog.svelte";
    import { DialogStore } from "$lib/state/dialog.svelte.js";
    import Paper from "$lib/components/Paper.svelte";
    import { readPaste } from "$lib/services/paste-service.js";
    import ContextMenu from "$lib/components/ContextMenu.svelte";
    import { ContextState } from "$lib/state/context.svelte";
    import InputButton from "$lib/components/InputButton.svelte";
    import { FileUp } from "lucide-svelte";
    import {
        addPDFContent,
        addURLContent,
        addBibTeXContent,
        addImageContent,
    } from "$lib/services/content-service.js";
    import { Store } from "$lib/state/database.svelte";
    import GridToggle from "$lib/components/GridToggle.svelte";
    let { data } = $props();
    let stack_id = $derived(data.stackID);
    let dragOverGrid = $state(false);
    let drag_id = $state(null);
    let selected_id = $state(null);
    let layout_grid = $state(true);
    $inspect(DialogStore.state);
    // Input Event Handlers
    async function handlePasteEvent(event: ClipboardEvent) {
        const payload = await readPaste(event);
        const selected_paper = Store.papers.find(
            (paper) => paper.id === selected_id,
        );
        switch (payload.type) {
            case "PDF": {
                DialogStore.start("Fetching PDF");
                await addPDFContent(stack_id, payload.content, selected_paper);
                break;
            }
            case "URL":
                await addURLContent(stack_id, payload.content, selected_paper);
                break;
            case "BibTeX":
                await addBibTeXContent(
                    stack_id,
                    payload.content,
                    selected_paper,
                );
                break;
            case "Image":
                await addImageContent(
                    stack_id,
                    payload.content,
                    selected_paper,
                );
                break;
        }
    }

    async function handleFileInput(event) {
        const selected_paper = Store.papers.find(
            (paper) => paper.id === selected_id,
        );
        const inputFiles = event.target.files;
        if (!inputFiles) return;
        if (selected_paper) {
            const newFile = inputFiles[0];
            if (newFile.type.includes("pdf")) {
                await addPDFContent(stack_id, newFile, selected_paper);
            }
            if (newFile.type.includes("image")) {
                await addImageContent(stack_id, newFile, selected_paper);
            }
        } else {
            for (const file of inputFiles) {
                if (file.type.includes("pdf")) {
                    await addPDFContent(stack_id, file, null);
                }
            }
        }
    }

    // Drag and Drop Handlers
    async function handleDrop(event) {
        let selected_paper = Store.papers.find((paper) => paper.id === drag_id);
        event.preventDefault();
        const droppedFiles = event.dataTransfer?.files;
        if (!droppedFiles) return;
        if (selected_paper) {
            const newFile = droppedFiles[0];
            if (newFile.type.includes("pdf")) {
                await addPDFContent(stack_id, newFile, selected_paper);
            }
            if (newFile.type.includes("image")) {
                await addImageContent(stack_id, newFile, selected_paper);
            }
        } else if (dragOverGrid) {
            //THIS IS WHY WE CANT MERGE BOTH HANDLERS
            for (const file of droppedFiles) {
                if (file.type.includes("pdf")) {
                    await addPDFContent(stack_id, file, null);
                }
            }
        }
        dragOverGrid = false;
        drag_id = null;
    }

    function handleContextMenu(event, paperId) {
        event.preventDefault();
        event.stopPropagation();
        selected_id = paperId;
        let paper = Store.papers.find((paper) => paper.id === paperId);
        ContextState.open_paper(event.clientX, event.clientY, paper);
    }

    function handleClick(event) {
        const clickedInside = event.target.closest(".card");
        const clickedContextMenu = event.target.closest(".context-menu");
        const clickedInput = event.target.closest("input");

        if (!clickedContextMenu) {
            ContextState.close();
        }

        if (!clickedInside && !clickedContextMenu && !clickedInput) {
            selected_id = null;
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

<ContextMenu />
<Dialog />

<div>
    <a href="/"> BACK </a>
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
>
    {#each Store.papers as paper (paper.id)}
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
            <Paper
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

    {#if Store.papers.length === 0}
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
