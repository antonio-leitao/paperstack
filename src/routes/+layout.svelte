<script>
    import "../app.css";
    import Notification from "./Notification.svelte";
    import { info } from "../stores.js";

    let notificationComponent;

    // Subscribe to the store to listen for updates
    info.subscribe((value) => {
        triggerNotification(value);
    });

    function triggerNotification(message) {
        if (notificationComponent) {
            notificationComponent.triggerSnackbar(message, "success");
        }
    }
</script>

<div class="app">
    <main>
        <slot />
    </main>
</div>

<Notification bind:this={notificationComponent} />
