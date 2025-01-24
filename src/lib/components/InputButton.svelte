<script>
    let { handleFileInput } = $props();
    import { FilePlus2 } from "lucide-svelte";
    let fileInput; // reference to the hidden file input

    const triggerFileInput = () => {
        fileInput.click();
    };
</script>

<div class="file-input-wrapper">
    <input
        bind:this={fileInput}
        type="file"
        accept="application/pdf"
        onchange={handleFileInput}
        class="hidden-input"
    />

    <button type="button" class="custom-file-button" onclick={triggerFileInput}>
        <FilePlus2 size={18} />
        <span>Add</span>
    </button>
</div>

<style>
    .file-input-wrapper {
        display: inline-block;
    }

    .hidden-input {
        display: none;
    }

    .custom-file-button {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 16px;
        background-color: var(--accent-color-light);
        color: white;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        font-size: 1rem;
        transition: background-color 0.2s;
    }

    .custom-file-button:hover {
        background-color: var(--accent-color);
    }
</style>
