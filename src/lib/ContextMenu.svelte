<script>
    import { onMount } from "svelte";
    import { Command } from "@tauri-apps/plugin-shell";
    import { ContextState } from "$lib/state/context.svelte";
    import { Store } from "$lib/state/database.svelte";
    import { DialogStore } from "$lib/state/dialog.svelte";
    import Cite from "citation-js";
    import {
        Trash2,
        PencilLine,
        SmilePlus,
        Blend,
        Paperclip,
        FileText,
        GraduationCap,
        ChevronRight,
        ExternalLink,
        BookCopy,
        FilePlus2,
        CircleX,
    } from "lucide-svelte";
    //let { show = $bindable(), x, y, paper, handleDelete } = $props();
    let link = $derived(ContextState.paper.url || ContextState.paper.bib.URL);
    let showExportSubmenu = $state(false);
    $inspect(Store.currentStackId);
    let otherStacks = $derived(
        Store.stacks.filter((s) => s.id !== Store.currentStackId),
    );

    function handleCopyBibTeX() {
        const cite = new Cite(ContextState.paper.bib);
        let bibtex = cite.format("bibtex");
        navigator.clipboard.writeText(bibtex);
        ContextState.show = false;
    }

    function handleCopyCite() {
        navigator.clipboard.writeText(ContextState.paper.bib.id);
        ContextState.show = false;
    }

    async function openPDF() {
        if (!ContextState.paper.pdf) return;
        try {
            await Command.create("open-pdf", [
                ContextState.paper.pdf,
            ]).execute();
        } catch (error) {
            console.error("Failed to open PDF:", error);
        }
        ContextState.close();
    }

    async function handleSendToStack(stackId) {
        if (!ContextState.paper) {
            return;
        }
        const { id, ...paper } = ContextState.paper;
        await Store.createPaper(stackId, paper);
        ContextState.close();
    }
    async function handleDeleteStack() {
        if (ContextState.stack) {
            Store.deleteStack(ContextState.stack.id);
        }
        ContextState.close();
    }
    async function handleRenameStack() {
        if (!ContextState.stack) return;
        ContextState.stackRename(ContextState.stack.id);
        ContextState.close();
    }
    async function handleDuplicateStack() {
        if (!ContextState.stack) return;
        Store.duplicateStack(ContextState.stack.id);
        ContextState.close();
    }
    // Add this new function
    async function handlePaperRemove() {
        if (!ContextState.paper) return;
        if (
            Store.currentStackId === "unsorted" ||
            Store.currentStackId === "all"
        ) {
            if (
                !(await DialogStore.confirm(
                    "Confirm Delete",
                    `Are you sure you want to delete ${ContextState.paper.bib.title}?`,
                ))
            )
                return;
            // In unsorted stack - delete permanently
            await Store.deletePaper(ContextState.paper.id);
        } else {
            // In regular stack - move to unsorted
            await Store.removePaperFromStack(
                ContextState.paper.id,
                Store.currentStackId,
            );
        }
        ContextState.close();
    }
    function handleStackIconChange() {}
    function handleStackMerge() {}
    function handleNewPaperInput() {}
</script>

{#if ContextState.show}
    <div
        class="context-menu"
        style="position: fixed; left: {ContextState.x}px; top: {ContextState.y}px;"
    >
        {#if ContextState.paper}
            {#if link}
                <a class="menu-item" href={link} target="_blank"
                    >Open Link<ExternalLink size={18} /></a
                >
            {/if}
            {#if ContextState.paper.pdf}
                <div class="menu-item" onclick={openPDF}>
                    Open PDF <FileText size={18} />
                </div>
            {/if}
            <div class="menu-item">
                Add Notes<Paperclip size={18} />
            </div>
            <div class="menu-item" onclick={handleCopyCite}>
                Cite<GraduationCap size={18} />
            </div>
            <div class="menu-item" onclick={handleCopyBibTeX}>
                Copy BibTeX<BookCopy size={18} />
            </div>
            {#if otherStacks.length > 0}
                <div
                    class="menu-item has-submenu"
                    onmouseenter={() => (showExportSubmenu = true)}
                    onmouseleave={() => (showExportSubmenu = false)}
                >
                    Add to <ChevronRight size={18} />
                    {#if showExportSubmenu}
                        <div class="submenu">
                            {#each otherStacks as stack}
                                <div
                                    class="menu-item"
                                    onclick={() => handleSendToStack(stack.id)}
                                >
                                    {stack.name}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
            <div class="separator"></div>
            {#if Store.currentStackId === "unsorted" || Store.currentStackId === "all"}
                <div class="menu-item delete" onclick={handlePaperRemove}>
                    Delete
                    <Trash2 size={18} />
                </div>
            {:else}
                <div class="menu-item delete" onclick={handlePaperRemove}>
                    Remove
                    <CircleX size={18} />
                </div>
            {/if}
        {:else if ContextState.stack}
            <div class="menu-item" onclick={handleRenameStack}>
                Rename<PencilLine size={18} />
            </div>
            <div class="menu-item" onclick={handleStackIconChange}>
                Change Icon<SmilePlus size={18} />
            </div>
            <div class="separator"></div>
            <div class="menu-item" onclick={handleDuplicateStack}>
                Duplicate<BookCopy size={18} />
            </div>
            <div class="menu-item" onclick={handleStackMerge}>
                Merge With<Blend size={18} />
            </div>
            <div class="separator"></div>
            <div class="menu-item delete" onclick={handleDeleteStack}>
                Delete<Trash2 size={18} />
            </div>
        {:else}
            <div class="menu-item" onclick={handleNewPaperInput}>
                Add<FilePlus2 size={18} />
            </div>
        {/if}
    </div>
{/if}

<style>
    .context-menu {
        background: white;
        border: 1px solid var(--surfaces);
        padding: 3px 4px;
        box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.2);
        min-width: 175px;
        z-index: 1000;
        position: relative;
        border-radius: 5px;
        box-shadow: var(--perfect-shadow);
    }

    .menu-item {
        color: var(--text-color);
        cursor: pointer;
        position: relative;
        display: flex;
        justify-content: space-between;
        align-items: center;
        transition: all 0.3s ease;
        padding: 6px 6px;
        border-radius: 3px;
    }

    .menu-item:hover {
        background-color: var(--surfaces);
    }

    .menu-item.delete:hover {
        background-color: #ffebee;
        color: #d32f2f;
    }

    .has-submenu {
        position: relative;
    }

    .submenu {
        position: absolute;
        left: 100%;
        top: 0;
        background: white;
        border: 1px solid var(--surfaces);
        border-radius: 3px;
        box-shadow: var(--perfect-shadow);
        min-width: 150px;
        z-index: 1001;
        padding: 3px 3px;
    }
    .separator {
        height: 1.3px;
        background-color: var(--surfaces);
        margin: 4px 0;
        border-radius: 10px;
    }
</style>
