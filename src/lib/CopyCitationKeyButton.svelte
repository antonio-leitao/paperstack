<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import KeyRound from "@lucide/svelte/icons/key-round";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import { invoke } from "@tauri-apps/api/core";
  import { copyToClipboard } from "./copyToClipboard";
  import type { BibtexPreview } from "./types";

  let {
    bibtex,
    label = "Copy citation key",
    ariaLabel = undefined,
    menuItem = false,
  }: {
    bibtex: string;
    label?: string;
    ariaLabel?: string;
    menuItem?: boolean;
  } = $props();

  let status = $state<"idle" | "copying" | "copied" | "failed">("idle");

  async function copy(event: MouseEvent) {
    event.stopPropagation();
    if (status === "copying") return;
    status = "copying";
    try {
      const preview = await invoke<BibtexPreview>("preview_bibtex", { bibtex });
      await copyToClipboard(preview.citationKey);
      status = "copied";
      window.setTimeout(() => {
        if (status === "copied") status = "idle";
      }, 1500);
    } catch {
      status = "failed";
    }
  }
</script>

<button
  class:paper-btn={!menuItem}
  type="button"
  role={menuItem ? "menuitem" : undefined}
  aria-label={ariaLabel}
  onclick={copy}
  disabled={status === "copying"}
>
  {#if status === "copying"}
    <LoaderCircle class="spin-icon" size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "failed"}
    <TriangleAlert size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "copied"}
    <Check size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else}
    <KeyRound size={16} strokeWidth={1.8} aria-hidden="true" />
  {/if}
  <span>
    {status === "copying"
      ? "Copying…"
      : status === "failed"
        ? "Copy failed"
        : status === "copied"
          ? "Copied"
          : label}
  </span>
</button>
