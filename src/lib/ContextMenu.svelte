<script>
    export let x = 0;
    export let y = 0;
    export let show = false;
</script>

{#if show}
    <div 
        class="context-menu"
        style="position: fixed; left: {x}px; top: {y}px;"
    >
        <slot />
    </div>
{/if}

<style>
    .context-menu {
        background: white;
        border: 1px solid #ccc;
        padding: 4px 0;
        box-shadow: 2px 2px 5px rgba(0,0,0,0.2);
        min-width: 150px;
        z-index: 1000;
    }
</style> 