<script>
    import { onMount } from "svelte";

    export let inputValue = "";
    let isVisible = false;

    function handleKeydown(event) {
        // Check for Cmd+K (Mac) or Ctrl+K (Windows/Linux)
        if (!isVisible && event.key === "s") {
            event.preventDefault();
            isVisible = true;
        }
    }

    function handleInputKeydown(event) {
        if (event.key === "Enter" || event.key === "Escape") {
            isVisible = false;
        }
    }

    onMount(() => {
        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
        };
    });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isVisible}
    <div class="input-wrapper">
        <input
            bind:value={inputValue}
            on:keydown={handleInputKeydown}
            placeholder="Search your stack"
            autofocus
        />
    </div>
{/if}

<style>
    .input-wrapper {
        position: fixed;
        top: 40%;
        left: 50%;
        transform: translate(-50%, -50%);
        z-index: 1000;
    }

    input {
        padding: 10px;
        font-size: 16px;
        width: 400px;
        color: var(--base);
        height: 60px;
        text-align: center;
        border: none;
        outline: none;
        background-color: rgba(255, 255, 255, 0.9);
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
</style>
