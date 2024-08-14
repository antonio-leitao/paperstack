<script>
    import { fade, scale } from "svelte/transition";
    import { quintOut } from "svelte/easing";

    let message = "";
    let type = "default"; // 'default', 'success', or 'error'
    let duration = 5000;
    let visible = false;

    // Define the triggerSnackbar function directly
    export function triggerSnackbar(
        messageText,
        messageType,
        messageDuration = 5000,
    ) {
        message = messageText;
        type = messageType;
        duration = messageDuration;
        showSnackbar();
    }

    async function showSnackbar() {
        visible = true;

        setTimeout(() => {
            closeSnackbar();
        }, duration);
    }

    function closeSnackbar() {
        visible = false;
    }

    function dismiss() {
        closeSnackbar();
    }

</script>

{#if visible}
    <div
        class="snackbar {type}"
        in:scale={{ duration: 250, easing: quintOut }}
        out:fade={{ duration: 300 }}
    >
        <span class="message">{message}</span>
        <button class="dismiss" on:click={dismiss}>×</button>
    </div>
{/if}

<style>
    .snackbar {
        position: fixed;
        bottom: 20px;
        left: 50%;
        transform: translateX(-50%);
        min-width: 250px;
        max-width: 90%;
        padding: 12px 24px;
        border-radius: 4px;
        box-shadow:
            0 3px 6px rgba(0, 0, 0 0.16),
            0 3px 6px rgba(0, 0, 0, 0.23);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .message {
        color: white;
        margin-right: 20px;
    }

    .dismiss {
        background: none;
        border: none;
        color: white;
        font-size: 20px;
        cursor: pointer;
        padding: 0;
    }
    .error {
        background-color: var(--error);
    }
    .success {
        background-color: #98dca4;
    }
    .warning {
        background-color: #f3e19c;
    }
    .default {
        background-color: white;
    }
</style>
