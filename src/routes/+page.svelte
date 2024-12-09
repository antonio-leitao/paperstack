<script lang="ts">
    import { flip } from 'svelte/animate';
    import { quintOut } from 'svelte/easing';
    import ProgressStep from "$lib/ProgressStep.svelte";
    import FileItem from "$lib/FileItem.svelte";
    import ConfirmDialog from "$lib/ConfirmDialog.svelte";
    import { extractTextFromPDF } from "$lib/services/pdf-service";
    import { extractBibFromPDF } from "$lib/services/ai-service";
    import { parseBibEntry } from "$lib/services/bib-service";
    import { readPaste } from "$lib/services/paste-service.js";
    import { LoadingState } from "$lib/state/loading.svelte";
    import ContextMenu from "$lib/ContextMenu.svelte";
    import { createFile, updateFile, deleteFile } from "$lib/state/database.svelte";

    // State
    let { data } = $props();
    let files = $state(data.files);
    let dragOverGrid = $state(false);
    let drag_id = $state(null);
    let selected_id = $state(null);
    let confirmationState = $state(null);
    let contextMenuX = $state(0);
    let contextMenuY = $state(0);
    let showContextMenu = $state(false);

    // Content Creation Handlers
    async function handlePDFContent(pdfFile, target_id: string) {
        LoadingState.start('Extracting PDF');
        try {
            const { text, pageCount: pages } = await extractTextFromPDF(pdfFile);
            LoadingState.lap('Asking AI');
            const bibtex = await extractBibFromPDF(text);
            const entry = parseBibEntry(bibtex);
            
            if (target_id) {
                files = await updateFile(target_id, { pages, ...entry });
            } else {
                files = await createFile({ pages, ...entry });
            }
        } catch (error) {
            console.error('Error processing PDF:', error);
        } finally {
            LoadingState.stop();
        }
    }

    async function handleBibTeXContent(bibtex: string, target_id: string) {
        const entry = parseBibEntry(bibtex);
        if (target_id) {
            await confirmUpdate({
                title: "Update Bibliography",
                message: `Update bibliography information?`,
                action: async () => {
                    files = await updateFile(target_id, entry);
                }
            });
        } else {
            files = await createFile(entry);
        }
    }

    async function handleURLContent(url: string, target_id: string) {
        if (target_id) {
            await confirmUpdate({
                title: "Update URL",
                message: `Update URL for this entry?`,
                action: async () => {
                    files = await updateFile(target_id, { url });
                }
            });
        }
    }

    // Input Event Handlers
    async function handlePasteEvent(event: ClipboardEvent) {
        const payload = await readPaste(event);
        switch (payload.type) {
            case 'PDF': {
                LoadingState.start('Fetching PDF');
                try {
                    const response = await fetch(payload.content);
                    const pdfBlob = await response.blob();
                    const pdfFile = new File([pdfBlob], 'pasted.pdf', { type: 'application/pdf' });
                    await handlePDFContent(pdfFile, selected_id);
                } catch (error) {
                    console.error('Error fetching PDF from URL:', error);
                }
                break;
            }
            case 'URL':
                await handleURLContent(payload.content, selected_id);
                break;
            case 'BibTeX':
                await handleBibTeXContent(payload.content, selected_id);
                break;
            case 'Image':
                // Handle image paste if needed
                break;
        }
    }

    async function handleFileInput(event) {
        const targetId = selected_id;
        const inputFiles = event.target.files;
        if (!inputFiles) return;
        if (targetId) {
            const newFile = inputFiles[0];
            await handlePDFContent(newFile, targetId);
        } else {
            for (const file of inputFiles) {
                if (file.type.includes('pdf')) {
                    await handlePDFContent(file, null);
                }
            }
        }
    }

    // Drag and Drop Handlers
    async function handleDrop(event) {
        const targetId = drag_id;
        event.preventDefault();
        const droppedFiles = event.dataTransfer?.files;
        if (!droppedFiles) return;
        if (targetId) {
            const newFile = droppedFiles[0];
            await handlePDFContent(newFile, targetId);
        } else if (dragOverGrid) { //THIS IS WHY WE CANT MERGE BOTH HANDLERS
            for (const file of droppedFiles) {
                if (file.type.includes('pdf')) {
                    await handlePDFContent(file, null);
                }
            }
        }
        dragOverGrid = false;
        drag_id = null;
    }

    // Helper Functions
    async function confirmUpdate({ title, message, action }: {
        title: string;
        message: string;
        action: () => Promise<void>;
    }) {
        confirmationState = {
            title,
            message,
            onConfirm: async () => {
                confirmationState = null;
                await action();
            },
            onCancel: () => {
                confirmationState = null;
            }
        };
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

    function handleDelete() {
        if (selected_id) {
            confirmationState = {
                title: "Delete File",
                message: "Are you sure you want to delete this file?",
                onConfirm: async () => {
                    files = await deleteFile(selected_id);
                    selected_id = null;
                    confirmationState = null;
                    showContextMenu = false;
                },
                onCancel: () => {
                    confirmationState = null;
                    showContextMenu = false;
                }
            };
        }
    }

    function handleCopyBibTeX() {
        if (selected_file) {
            const bibtex = `@article{${selected_file.id},\n\ttitle={${selected_file.title}}\n}`;
            navigator.clipboard.writeText(bibtex);
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

<ContextMenu x={contextMenuX} y={contextMenuY} show={showContextMenu}>
    <div class="menu-item" onclick={handleCopyBibTeX}>Copy BibTeX</div>
    <div class="menu-item" onclick={handleDelete}>Delete</div>
</ContextMenu>

<div>
    <input type="file" accept="application/pdf" onchange={handleFileInput} />

    <div class="progress">
        <ProgressStep label={LoadingState.what} isCompleted={!LoadingState.is_loading} />
    </div>
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
    {#each files as file, index (file.id)}
        <div 
            class="card" 
            role="gridcell"
            data-file-id={file.id}
            onclick={() => {selected_id = file.id}}
            oncontextmenu={(event) => handleContextMenu(event, file.id)}
            onkeydown={(event) => {if (event.key === 'Enter' || event.key === ' ') {selected_id = file.id}}}
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

{#if confirmationState}
    <ConfirmDialog
        title={confirmationState.title}
        message={confirmationState.message}
        confirmText={confirmationState.confirmText}
        cancelText={confirmationState.cancelText}
        handleConfirm={confirmationState.onConfirm}
        handleCancel={confirmationState.onCancel}
    />
{/if}

<style>
    .file-grid {
        user-select: none;
    }

    :global(.menu-item) {
        padding: 8px 16px;
        cursor: pointer;
    }

    :global(.menu-item:hover) {
        background-color: #f0f0f0;
    }
</style>

