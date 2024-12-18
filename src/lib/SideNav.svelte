<script lang="ts">
    import { Navbar } from "$lib/state/database.svelte.js";
    import { fly } from "svelte/transition";
    import { quintInOut } from "svelte/easing";
    import { ContextState } from "$lib/state/context.svelte";
    import {
        ArrowLeft,
        Layers,
        Plus,
        EllipsisVertical,
        SquareLibrary,
    } from "lucide-svelte";
    import { goto } from "$app/navigation";

    let { isOpen = $bindable(), initialId } = $props();
    let currentStackId = $state(initialId);
    let stacks = [
        { id: 1, name: "topological data analysis" },
        { id: 2, name: "To Read" },
        { id: 3, name: "Unsorted" },
        { id: 4, name: "Machine Learning" },
    ];
    function trimText(text: string) {
        return text.length > 20 ? text.slice(0, 20) + "..." : text;
    }
    function handleContextMenu(event, stackId) {
        event.preventDefault();
        event.stopPropagation();
        currentStackId = stackId;
        let stack = stacks.find((stack) => stack.id === stackId);
        ContextState.open_stack(event.clientX, event.clientY, stack);
    }
    function handleClick(stackId) {
        currentStackId = stackId;
        goto(`/${currentStackId}`);
    }
</script>

<div class="header">
    <button class="toggle-btn" onclick={() => (isOpen = !isOpen)}>
        <div class="icon">
            {#if isOpen}
                <ArrowLeft size={18} />
            {:else}
                <Layers size={18} /> {Navbar.stacks.length}
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
            Stacks <div class="icon"><Plus size={18} /></div>
        </div>
        <div class="stack-tray">
            {#each stacks as stack (stack.id)}
                <div
                    class="list-item"
                    onclick={() => handleClick(stack.id)}
                    oncontextmenu={(event) =>
                        handleContextMenu(event, stack.id)}
                    class:active={currentStackId === stack.id}
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
    button {
        background-color: transparent;
        border: none;
        cursor: pointer;
    }
</style>
