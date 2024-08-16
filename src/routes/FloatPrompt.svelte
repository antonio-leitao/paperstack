<script>
    import { onMount } from "svelte";
    import { createEventDispatcher } from "svelte";

    // Create an event dispatcher
    const dispatch = createEventDispatcher();

    function submission() {
        dispatch("submission", { value: inputValue });
    }

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
        switch (event.key) {
            case "Enter":
                submission();
                isVisible = false;
                return [];
            case "Escape":
                isVisible = false;
                return [];
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
        border: solid 1px rgb(229, 231, 235);
        outline: none;
        background-color: rgba(255, 255, 255, 0.9);
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
</style>
