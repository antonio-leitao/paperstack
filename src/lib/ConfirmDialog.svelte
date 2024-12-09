<script>
    let { title = "Confirm Action", message = "", confirmText = "Confirm", cancelText = "Cancel", handleConfirm, handleCancel } = $props();
</script>

<div class="modal-overlay">
    <div class="modal" role="dialog" aria-modal="true">
        <h2>{title}</h2>
        <p>{message}</p>
        <div class="button-group">
            <button onclick={handleCancel}>{cancelText}</button>
            <button onclick={handleConfirm} class="primary">{confirmText}</button>
        </div>
    </div>
</div>

<style>
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal {
        background: white;
        padding: 20px;
        border-radius: 8px;
        max-width: 400px;
        width: 90%;
    }

    .button-group {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 20px;
    }

    button.primary {
        background: var(--primary-color, #4CAF50);
        color: white;
    }
</style>
