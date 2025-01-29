<script>
    let {
        tooltip,
        enableTooltip = true, // New prop to toggle tooltip functionality
    } = $props();
    import { onDestroy } from "svelte";
  import { quintInOut } from "svelte/easing";
  import {fly} from "svelte/transition";

    let showTooltip = $state(false);
    let tooltipTimeout;


    function handleMouseLeave() {
        showTooltip = false;
    }

    function handleMouseEnter(){
        if (!enableTooltip){return}
        showTooltip = true
    }
    onDestroy(() => {
        clearTimeout(tooltipTimeout);
    });
</script>

<div
    class="tooltip-container"
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
>
    <slot />

    {#if showTooltip}
        <div transition:fly={{x:-8,duration:100, opacity:0.7, easing:quintInOut}}
        class="info tooltip" role="tooltip">
            {tooltip}
        </div>
    {/if}
</div>

<style>
    .tooltip-container {
        position: relative;
        display: inline-block;
    }

    .tooltip {
        position: absolute;
        top: 50%;
        left: 100%;
        border:1px solid var(--surfaces);
        transform: translateY(-50%);
        margin-left: 0.5rem;
        background-color:white; 
        padding: 0.3rem 0.6rem;
        border-radius: 0.3rem;
        white-space: nowrap;
        z-index: 1002;
    }
</style>
