<script lang="ts">
    import { fly } from "svelte/transition";
    import { quintInOut } from "svelte/easing";
    import { ContextState } from "$lib/state/context.svelte";
    import {
        Store,
        reorderStacks,
        createStack,
    } from "$lib/state/database.svelte";
    import {
        ArrowLeft,
        Layers,
        Plus,
        EllipsisVertical,
        SquareLibrary,
    } from "lucide-svelte";
    import { goto } from "$app/navigation";

    let { isOpen = $bindable() } = $props();

    let draggedItem = $state(null);
    let hoveredItem = $state(null);

    function trimText(text: string) {
        return text.length > 20 ? text.slice(0, 20) + "..." : text;
    }

    async function newUnnamedStack() {
        const name = `Stack #${Store.stacks.length + 1}`;
        let stackId = await createStack(name);
        goto(`/stack/${stackId}`);
    }
    function handleContextMenu(event, stackId) {
        event.preventDefault();
        event.stopPropagation();
        let stack = Store.stacks.find((stack) => stack.id === stackId);
        ContextState.open_stack(event.clientX, event.clientY, stack);
    }

    function handleClick(stackId) {
        goto(`/stack/${stackId}`);
    }

    function handleDragStart(event, stackId) {
        draggedItem = stackId;
        event.dataTransfer.effectAllowed = "move";
    }

    function handleDragOver(event, stackId) {
        event.preventDefault();
        if (draggedItem === stackId) return;
        hoveredItem = stackId;
        event.dataTransfer.dropEffect = "move";
    }

    function handleDragLeave() {
        hoveredItem = null;
    }

    function handleDrop(event, stackId) {
        event.preventDefault();
        if (draggedItem === null || draggedItem === stackId) {
            hoveredItem = null;
            return; // Avoid reordering if dropped in the same place
        }
        reorderStacks(draggedItem, stackId);
        draggedItem = null;
        hoveredItem = null;
    }
</script>

<div class="header">
    <button class="toggle-btn" onclick={() => (isOpen = !isOpen)}>
        <div class="icon">
            {#if isOpen}
                <ArrowLeft size={18} />
            {:else}
                <Layers size={18} /> {Store.stacks.length}
            {/if}
        </div>
    </button>
</div>
{#if isOpen}
    <div
        class="nav-list"
        class:open={isOpen}
        in:fly={{ x: -250, duration: 300, easing: quintInOut }}
        out:fly={{ x: -250, duration: 300, easing: quintInOut }}
    >
        <div class="info">
            Stacks <div class="icon">
                <button onclick={newUnnamedStack}><Plus size={18} /></button>
            </div>
        </div>
        <div class="stack-tray">
            {#each Store.stacks as stack (stack.id)}
                <div
                    draggable="true"
                    class="list-item"
                    class:dragging={draggedItem === stack.id}
                    class:hovered={hoveredItem === stack.id}
                    onclick={() => handleClick(stack.id)}
                    oncontextmenu={(event) =>
                        handleContextMenu(event, stack.id)}
                    ondragstart={(event) => handleDragStart(event, stack.id)}
                    ondragover={(event) => handleDragOver(event, stack.id)}
                    ondragleave={handleDragLeave}
                    ondrop={(event) => handleDrop(event, stack.id)}
                    class:active={Store.currentStackId === stack.id}
                >
                    <div class="icon">
                        <SquareLibrary size={18} />
                    </div>
                    <div class="text">{trimText(stack.name)}</div>
                    <div
                        class="icon"
                        onclick={(e) => handleContextMenu(e, stack.id)}
                    >
                        <EllipsisVertical size={18} />
                    </div>
                </div>
            {/each}
        </div>
    </div>
{/if}

<style>
    .nav-list {
        position: absolute;
        top: 3rem;
        left: 0;
        height: 100vh;
        width: 15rem;
        overflow-x: hidden;
        z-index: 10; /* Ensure it's above other content */
        display: flex;
        flex-direction: column;
        overflow: hidden;
        padding: 0.5rem;
    }
    .info {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-left: 1rem;
        padding: 0.3rem 0.7rem;
    }
    .stack-tray {
        display: flex;
        flex-direction: column;
        align-items: left;
        padding: 0; /* Remove padding */
        margin: 0; /* Remove margins */
    }
    .list-item {
        padding: 0.7rem;
        border-radius: 0.7rem;
        cursor: pointer;
        display: flex;
        align-items: center;
        text-decoration: none;
        justify-content: space-between;
        position: relative;
        color: inherit;
    }
    .list-item:hover {
        background-color: var(--surfaces);
    }
    .active {
        background-color: var(--accent-color-translucent);
    }
    .list-item.active:hover {
        background-color: var(--accent-color-translucent);
    }
    .icon {
        border-radius: 50%;
        display: grid;
        place-items: center;
        width: 1.5rem;
        height: 1.5rem; /* Or as needed */
    }
    .list-item .icon {
        display: grid;
        place-items: center;
        width: 1.5rem;
        height: 1.5rem; /* Or as needed */
    }
    .icon:hover {
        background-color: var(--surfaces);
    }

    .list-item .text {
        flex-grow: 1;
        white-space: nowrap;
        overflow: hidden;
        user-select: none;
    }
    .list-item.dragging {
        opacity: 0.5;
    }

    .list-item.hovered::after {
        content: "";
        position: absolute;
        bottom: 0;
        left: 50%;
        width: 80%;
        height: 2px;
        background-color: var(--accent-color);
        transform: translateX(-50%);
        transition: height 0.2s ease-in-out;
    }
    button {
        background-color: transparent;
        border: none;
        cursor: pointer;
    }
</style>
