<!-- SideNav.svelte -->
<script lang="ts">
    import { flip } from "svelte/animate";
    import { quintInOut } from "svelte/easing";
    import { ContextState } from "$lib/state/context.svelte";
    import {
        Store,
        getUnsortedStack,
        reorderStacks,
        createStack,
        updateStack,
    } from "$lib/state/database.svelte";
    // Add Search to imports
    import {
        ArrowLeft,
        Layers,
        Plus,
        Ellipsis,
        Files,
        FileQuestion,
        SquareLibrary,
        Search, // Add this
    } from "lucide-svelte";
    import TooltipButton from "$lib/components/TooltipButton.svelte";
    import { goto } from "$app/navigation";
    import { createSwapy } from "swapy";
    import { onDestroy } from "svelte";
    import { Icons } from "$lib/utils/stackIcons.js";

    let { isOpen = $bindable() } = $props();
    let editingStackId = $state(null);
    let newStackName = $state("");
    let container = $state(null);
    let swapy = $state(null);
    let finalOrder = $state(null);
    
    ContextState.stackRename = startEditing;

    // Initialize swapy once when the container is available
    $effect(() => {
        if (container && !swapy) {
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

    // Cleanup only on component destroy
    onDestroy(() => {
        if (swapy) {
            reorderStacks(finalOrder);
            swapy.destroy();
            swapy = null;
            // Update the order immediately when changed
        }
    });

    const trimText = (text: string) => text.length > 20 ? text.slice(0, 20) + "..." : text;

    async function newUnnamedStack() {
        const name = `Stack #${Store.stacks.length + 1}`;
        const stackId = await createStack(name);
        goto(`/stack/${stackId}`);
    }

    function handleContextMenu(event, stackId) {
        event.preventDefault();
        event.stopPropagation();
        const stack = Store.stacks.find((stack) => stack.id === stackId);
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
        setTimeout(() => {
            const inputField = document.getElementById(`input-${stackId}`);
            inputField?.focus();
            inputField?.select();
        }, 0);
    }

    async function handleRename(stackId) {
        await updateStack(stackId, { name: newStackName });
        editingStackId = null;
        newStackName = "";
    }

    function handleRenameInputBlur(stackId) {
        const originalName = Store.stacks.find((s) => s.id === stackId)?.name || "";
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
    let unsortedActive = $derived(getUnsortedStack().papers.length>0);
</script>

<nav class="nav-container" class:collapsed={!isOpen}>

    <div class="nav-content">
    <div class="section-header">
        <button class="toggle-btn" onclick={() => isOpen = !isOpen}>
            {#if isOpen}
                <ArrowLeft size={18} />
            {:else}
                <Layers size={18} />
            {/if}
        </button>
        
        {#if isOpen}
            <Search size={18} />
        {/if}
    </div>
        <!-- Fixed Items Section -->
        <div class="nav-section">
            <div 
                class="nav-item"
                class:active={Store.currentStackId === "all"}
                onclick={() => handleClick("all")}
            >
                <TooltipButton tooltip="All" enableTooltip={!isOpen}>
                <div class="icon">
                    <Files size={18}/>
                </div>
                </TooltipButton>
                {#if isOpen}
                    <span class="label">All</span>
                {/if}
            </div>

                <div 
                    class="nav-item"
                    class:active={Store.currentStackId === "unsorted"}
                    onclick={() => {if(unsortedActive) {handleClick("unsorted")}}}
                    class:disabled={!unsortedActive}
                >
                <TooltipButton tooltip="Unsorted" enableTooltip={!isOpen}>
                    <div class="icon">
                        <FileQuestion size={18}/>
                    </div>
                    </TooltipButton>
                    {#if isOpen}
                        <span class="label">Unsorted</span>
                    {/if}
                </div>
        </div>

        <!-- Stacks Section Header -->
        <div class="section-header">
            {#if isOpen}
                <span class="label info">Stacks</span>
            {/if}
            <div class="icon plus-icon" onclick={newUnnamedStack}>
                <TooltipButton tooltip="New stack" enableTooltip={!isOpen}>
                    <Plus size={18}/>
                </TooltipButton>
            </div>
        </div>

        <!-- Stacks List -->
        <div class="nav-section" bind:this={container}>
            {#each Store.stacks as stack (stack.id)}
                <div
                    class="slot"
                    data-swapy-slot={stack.id}
                    animate:flip={{ duration: 150, easing: quintInOut }}
                >
                    <div
                        data-swapy-item={stack.id}
                        class="nav-item"
                        class:active={Store.currentStackId === stack.id}
                        onclick={() => handleClick(stack.id)}
                        oncontextmenu={(event) => handleContextMenu(event, stack.id)}
                    >

                        <TooltipButton tooltip={stack.name} enableTooltip={!isOpen}>
                            <div class="icon">
                                {#if stack.icon && Icons[stack.icon]}
                                    <svelte:component this={Icons[stack.icon]} size={18} />
                                {:else}
                                    <SquareLibrary size={18} />
                                {/if}
                            </div>
                        </TooltipButton>
                        
                        {#if isOpen}
                            {#if editingStackId === stack.id}
                                <input
                                    id="input-{stack.id}"
                                    bind:value={newStackName}
                                    onblur={() => handleRenameInputBlur(stack.id)}
                                    onkeydown={(event) => handleRenameInputKeydown(event, stack.id)}
                                    class="edit-input"
                                />
                            {:else}
                                <span class="label">{trimText(stack.name)}</span>
                            {/if}

                            <div
                                class="icon action-icon"
                                onclick={(e) => {
                                    e.stopPropagation();
                                    handleContextMenu(e, stack.id);
                                }}
                            >
                                <Ellipsis size={18} />
                            </div>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>
    </div>
</nav>

<style>
    .nav-container {
        position: fixed;
        left: 0;
        top: 0;
        height: 100vh;
        background-color: white;
        width: 15rem;
        transition: width 0.3s cubic-bezier(0.86, 0, 0.07, 1);
        display: flex;
        flex-direction: column;
        z-index:100;
        box-shadow: 0 0 8px rgba(0, 0, 0, 0.05);
        --order:3;
        --shadow: calc(var(--order) * 1px);
        box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
            rgba(0, 0, 0, 0.25);
        box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
            rgba(0, 0, 0, 0.25);
    }

    .nav-container.collapsed {
        width: 3.5rem;
    }

    .toggle-btn {
        background: transparent;
        border: none;
        cursor: pointer;
        color: inherit;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.5rem;
        width: 2rem;
        height: 2rem;
    }

    .nav-content {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
        padding: 0.5rem;
    }

    .nav-section {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .nav-item {
        display: flex;
        align-items: center;
        padding: 0.5rem;
        border-radius: 0.5rem;
        cursor: pointer;
        background-color: white;
        gap: 0.25rem;
        transition: background-color 0.2s;
    }
    .nav-item.disabled{
        pointer-events:none;
        cursor:not-allowed;
        color:var(--shades)
    }

    .nav-item:hover {
        background-color: var(--surfaces);
    }
    .nav-container.collapsed .nav-item:hover{
        color:var(--accent-color);
        background-color:white;
    }

    .nav-item.active {
        background-color: var(--accent-color-translucent);
    }
    .nav-container.collapsed .nav-item{
        color:var(--shades)
    }
    .nav-container.collapsed .nav-item.active{
        color:var(--accent-color);
        background-color:white;
    }


    .icon {
        cursor:pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        min-width: 24px;
    }

    .label {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-size: 0.9rem;
    }
    .info{
        font-size:0.75rem;
    }

    .section-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.5rem;
        margin-top: 1rem;
    }

    .edit-input {
        flex: 1;
        border: none;
        outline: solid 2px var(--accent-color-light);
        border-radius: 0.3rem;
        background-color: transparent;
        color: inherit;
        font-size: 0.9rem;
        font-family: inherit;
        width:100%;
        margin:0;
    }

    .action-icon {
        opacity: 0;
        transition: opacity 0.2s;
    }

    .nav-item:hover .action-icon {
        opacity: 1;
    }

    .plus-icon {
        margin-left: auto;
    }
    .nav-container.collapsed .nav-item {
        min-width: 2rem; /* Ensure there's enough grab area when collapsed */
    }

    .nav-container.collapsed .slot {
        width: 100%;
    }

    /* Make sure the drag handle area is accessible in both states */
    [data-swapy-item] {
        width: 100%;
    }

    [data-swapy-item]:active {
        cursor: grabbing;
    }
</style>