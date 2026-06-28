<script lang="ts">
  import { copyToClipboard } from "./copyToClipboard";

  // Self-contained "Copy BibTeX" control. Owns its three visual states
  // (idle / copied / failed) so callers don't have to track per-reference copy
  // state. Stops click propagation so it can live inside hover popovers.
  let {
    bibtex,
    label = "Copy BibTeX",
    ariaLabel = undefined,
  }: {
    bibtex: string;
    label?: string;
    ariaLabel?: string;
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

<button type="button" aria-label={ariaLabel} onclick={copy}>
  {status === "failed" ? "Copy failed" : status === "copied" ? "Copied" : label}
</button>
