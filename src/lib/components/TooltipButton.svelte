<script>
    let {
        icon,
        tooltip,
        disabled = false,
        selected = false,
        onClick,
    } = $props();
    import { onDestroy } from "svelte";

    let showTooltip = $state(false);
    let tooltipTimeout;

    function handleMouseEnter() {
        tooltipTimeout = setTimeout(() => {
            showTooltip = true;
        }, 500);
    }

    function handleMouseLeave() {
        clearTimeout(tooltipTimeout);
        showTooltip = false;
    }

    onDestroy(() => {
        clearTimeout(tooltipTimeout);
    });

    const SvelteComponent = $derived(icon);
</script>

<div
    class="tooltip-container"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
>
    <button
        class="icon-button {selected ? 'selected' : ''}"
        {disabled}
        onclick={onClick}
        aria-label={tooltip}
    >
        <SvelteComponent size={18} />
    </button>

    {#if showTooltip}
        <div class="tooltip" role="tooltip">
            {tooltip}
        </div>
    {/if}
</div>

<style>
    .tooltip-container {
        position: relative;
        display: inline-block;
    }

    .icon-button {
        background-color: transparent;
        border: none;
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 0.5rem;
        color: inherit;
        display: grid;
        place-items: center;
    }

    .icon-button:hover:not([disabled]),
    .icon-button.selected {
        background-color: var(--surfaces);
    }

    .icon-button[disabled] {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .tooltip {
        position: absolute;
        bottom: -2rem;
        left: 50%;
        transform: translateX(-50%);
        background-color: var(--text-color);
        color: var(--background-color);
        padding: 0.3rem 0.6rem;
        border-radius: 0.3rem;
        font-size: 0.8rem;
        white-space: nowrap;
        z-index: 1002;
    }
</style>
