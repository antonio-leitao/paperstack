<script lang="ts">
  import { isTauri } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  // Imports the global reset + design tokens once for every window (the project
  // organizer and each standalone viewer all render under this root layout).
  import "../app.css";

  let { children } = $props();

  onMount(() => {
    const usesMacOSOverlay =
      isTauri() && /Macintosh|Mac OS X/.test(navigator.userAgent);
    document.documentElement.classList.toggle(
      "macos-titlebar-overlay",
      usesMacOSOverlay,
    );
    return () => {
      document.documentElement.classList.remove("macos-titlebar-overlay");
    };
  });
</script>

<div class="window-titlebar-drag-region" data-tauri-drag-region aria-hidden="true"></div>
{@render children()}
