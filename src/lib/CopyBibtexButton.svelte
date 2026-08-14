<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Copy from "@lucide/svelte/icons/copy";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
  import { copyToClipboard } from "./copyToClipboard";

  // Self-contained "Copy BibTeX" control. Owns its three visual states
  // (idle / copied / failed) so callers don't have to track per-reference copy
  // state. Stops click propagation so it can live inside hover popovers.
  let {
    bibtex,
    label = "Copy BibTeX",
    ariaLabel = undefined,
    menuItem = false,
  }: {
    bibtex: string;
    label?: string;
    ariaLabel?: string;
    menuItem?: boolean;
  } = $props();

  let status = $state<"idle" | "copied" | "failed">("idle");

  async function copy(event: MouseEvent) {
    event.stopPropagation();
    try {
      await copyToClipboard(bibtex);
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
>
  {#if status === "failed"}
    <TriangleAlert size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "copied"}
    <Check size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else}
    <Copy size={16} strokeWidth={1.8} aria-hidden="true" />
  {/if}
  <span>{status === "failed" ? "Copy failed" : status === "copied" ? "Copied" : label}</span>
</button>
