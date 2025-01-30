<script>
    import { Command } from "@tauri-apps/plugin-shell";
    import { ContextState } from "$lib/state/context.svelte";
    import { Store } from "$lib/state/database.svelte";
    import { DialogStore } from "$lib/state/dialog.svelte";
    import { categories, Icons } from "$lib/utils/stackIcons.js";
    import Cite from "citation-js";
    import {
        Trash2,
        PencilLine,
        SmilePlus,
        Blend,
        MessageSquareText,
        MessageSquarePlus,
        MessageSquareX,
        Sparkles,
        FileText,
        GraduationCap,
        ChevronRight,
        ExternalLink,
        BookCopy,
        FilePlus2,
        CircleX,
        ArrowLeft,
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
        const { id } = ContextState.paper;
        await Store.movePaper(stackId, id);
        ContextState.close();
    }
    async function handleDeleteStack() {
        if (ContextState.stack) {
            await Store.deleteStack(ContextState.stack.id);
            if (Store.currentStackId === ContextState.stack.id) {
                goto('/stack/all');
            }
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
    function handleStackMerge() {}
    function handleNewPaperInput() {}
    let showingIconSelector = $state(false);

    function handleStackIconChange(event) {
        event.stopPropagation();
        showingIconSelector = true;
    }

    $effect(() => {
    if (!ContextState.show) {
        showingIconSelector = false;
    }
    });
    function handleBackToMenu(event) {
    event.stopPropagation();
    showingIconSelector = false;
}

    async function selectIcon(iconName) {
        if (!ContextState.stack) return;
        await Store.updateStack(ContextState.stack.id, {icon:iconName});
        showingIconSelector = false;
        ContextState.close();
    }
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
            <div class="menu-item" onclick={handleCopyCite}>
                Cite<GraduationCap size={18} />
            </div>
            <div class="menu-item" onclick={handleCopyBibTeX}>
                Copy BibTeX<BookCopy size={18} />
            </div>
            <div class="separator"></div>
            {#if ContextState.paper.summary}
                <div class="menu-item">
                    Edit Note<MessageSquareText size={18} />
                </div>
                <div class="menu-item">
                    Remove Note<MessageSquareX size={18} />
                </div>
            {:else}
                <div class="menu-item">
                    Add Note<MessageSquarePlus size={18} />
                </div>
                <div class="menu-item">
                    Ai Note<Sparkles size={18} />
                </div>
            {/if}
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
            {#if showingIconSelector}
            <div class="icon-grid">
                <div class="menu-item back-button" onclick={handleBackToMenu}>
                    <ArrowLeft size={18} />Back
                </div>
                {#each categories as category}
                    <div class="category-section">
                        <div class="subinfo">{category.name}</div>
                        <div class="icon-options">
                            {#each Object.entries(category.icons) as [name, component]}
                                <div class="icon-option" onclick={() => selectIcon(name)}>
                                    <svelte:component this={component} size={14} />
                                </div>
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
            {:else}
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
        {/if}
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
        text-overflow: ellipsis;
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
    .icon-grid {
        display: flex;
        flex-direction: column;
        gap: 5px;
        padding: 4px;
    }

    .back-button {
        display: flex;
        align-items: center;
        gap: 5px;
        padding: 6px;
        border-bottom: 1px solid var(--surfaces);
        margin-bottom: 4px;
    }
    .category-section {
        margin-bottom: 8px;
    }

    .category-title {
        font-size: 0.8rem;
        color: var(--shades);
        padding: 4px 6px;
        margin-bottom: 4px;
    }

    .icon-options {
        display: grid;
        grid-template-columns: repeat(6, 1fr);
        gap: 5px;
    }

    .icon-option {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 5px;
        border-radius: 4px;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .icon-option:hover {
        background-color: var(--surfaces);
    }
</style>
