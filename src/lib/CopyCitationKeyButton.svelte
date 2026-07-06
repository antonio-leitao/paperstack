<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { copyToClipboard } from "./copyToClipboard";
  import type { BibtexPreview } from "./types";

  let {
    bibtex,
    ariaLabel = undefined,
    menuItem = false,
  }: {
    bibtex: string;
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
  class:eink-btn={!menuItem}
  type="button"
  role={menuItem ? "menuitem" : undefined}
  aria-label={ariaLabel}
  onclick={copy}
  disabled={status === "copying"}
>
  {status === "copying"
    ? "Copying…"
    : status === "failed"
      ? "Copy failed"
      : status === "copied"
        ? "Copied"
        : "Copy citation key"}
</button>
