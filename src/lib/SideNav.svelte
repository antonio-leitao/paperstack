<script lang="ts">
    import { flip } from "svelte/animate";
    import { quintInOut } from "svelte/easing";
    import { fly } from "svelte/transition";
    import { ContextState } from "$lib/state/context.svelte";
    import {
        Store,
        reorderStacks,
        createStack,
        updateStack,
    } from "$lib/state/database.svelte";
    import {
        ArrowLeft,
        Layers,
        Plus,
        EllipsisVertical,
        SquareLibrary,
    } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { createSwapy } from "swapy";
    import { onDestroy, onMount } from "svelte";

    let { isOpen = $bindable() } = $props();
    let editingStackId = $state(null);
    let newStackName = $state("");
    ContextState.stackRename = startEditing;
    let container = null;
    let swapy = null;
    let finalOrder = $state(null);

    // Handle initialization
    $effect(() => {
        if (isOpen && container && !swapy) {
            console.log("loading swapy");
            swapy = createSwapy(container, {
                autoScrollSpeed: 10,
                threshold: 0.5,
            });
            swapy.onSwapEnd((event) => {
                if (event.hasChanged) {
                    finalOrder = event.slotItemMap.asArray;
                }
            });
        }
    });

    // Handle cleanup
    $effect(() => {
        if (!isOpen && swapy) {
            swapy.destroy();
            swapy = null;
            if (finalOrder) {
                reorderStacks(finalOrder);
                finalOrder = null; // Reset finalOrder after processing
            }
        }
    });
    onDestroy(() => {
        swapy?.destroy();
        if (finalOrder) {
            reorderStacks(finalOrder); // Update backend on destroy
        }
    });

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
        if (editingStackId !== stackId) {
            goto(`/stack/${stackId}`);
        }
    }

    function startEditing(stackId) {
        editingStackId = stackId;
        newStackName = Store.stacks.find((s) => s.id === stackId)?.name || "";
        // Focus and select text in the input field after a slight delay
        setTimeout(() => {
            const inputField = document.getElementById(`input-${stackId}`);
            if (inputField) {
                inputField.focus();
                inputField.select();
            }
        }, 0);
    }

    async function handleRename(stackId) {
        await updateStack(stackId, newStackName);
        editingStackId = null;
        newStackName = "";
    }

    function handleRenameInputBlur(stackId) {
        // Prevent renaming if the new name is empty or the same as the old name
        let originalName =
            Store.stacks.find((s) => s.id === stackId)?.name || "";
        if (newStackName.trim() === "" || newStackName === originalName) {
            editingStackId = null;
            newStackName = "";
            return;
        }
        handleRename(stackId);
    }

    function handleRenameInputKeydown(event, stackId) {
        if (event.key === "Enter") {
            handleRenameInputBlur(stackId);
        } else if (event.key === "Escape") {
            editingStackId = null;
            newStackName = "";
        }
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
        <div class="stack-tray" bind:this={container}>
            {#each Store.stacks as stack (stack.id)}
                <div
                    class="slot"
                    data-swapy-slot={stack.id}
                    animate:flip={{ duration: 150, easing: quintInOut }}
                >
                    <div
                        data-swapy-item={stack.id}
                        data-stack-id={stack.id}
                        class="list-item"
                        onclick={() => handleClick(stack.id)}
                        oncontextmenu={(event) =>
                            handleContextMenu(event, stack.id)}
                        class:active={Store.currentStackId === stack.id}
                    >
                        <div class="icon">
                            <SquareLibrary size={18} />
                        </div>
                        {#if editingStackId === stack.id}
                            <input
                                id="input-{stack.id}"
                                bind:value={newStackName}
                                onblur={() => handleRenameInputBlur(stack.id)}
                                onkeydown={(event) =>
                                    handleRenameInputKeydown(event, stack.id)}
                                class="edit-input"
                            />
                        {:else}
                            <div class="text">
                                {trimText(stack.name)}
                            </div>
                        {/if}

                        <div
                            class="icon"
                            onclick={(e) => {
                                e.stopPropagation();
                                handleContextMenu(e, stack.id);
                            }}
                        >
                            <EllipsisVertical size={18} />
                        </div>
                    </div>
                </div>
            {/each}
        </div>
    </div>
{/if}

<!-- Same styles as before -->
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
        background-color: var(--platinum);
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
    .edit-input {
        border: none;
        width: 100%;
        outline: solid 2px var(--accent-color-light);
        border-radius: 0.3rem;
        background-color: transparent;
        color: inherit;
        font-size: inherit;
        font-family: inherit;
    }
    .stack-tray [data-swapy-item] {
        touch-action: none;
    }
    [data-swapy-item] {
        cursor: pointer;
    }
</style>
