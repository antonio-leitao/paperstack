<script lang="ts">
  import type { AnalysisStatus, LibraryDocument } from "./types";

  let {
    document,
    analysisState,
    projectActionLabel,
    projectActionDisabled = false,
    onopen,
    onrename,
    onlinkbibtex,
    onunlink,
    onanalyze,
    onprojectaction,
    ondelete,
  }: {
    document: LibraryDocument;
    analysisState?: AnalysisStatus;
    projectActionLabel: string;
    projectActionDisabled?: boolean;
    onopen: () => void;
    onrename: () => void;
    onlinkbibtex: () => void;
    onunlink: () => void;
    onanalyze: () => void;
    onprojectaction: () => void;
    ondelete: () => void;
  } = $props();
</script>

<div class="items">
  <button type="button" role="menuitem" onclick={onopen}>Open</button>
  <button type="button" role="menuitem" onclick={onrename}>Rename</button>
  {#if document.referenceId}
    <button type="button" role="menuitem" onclick={onunlink}>
      Unlink reference
    </button>
  {:else}
    <button type="button" role="menuitem" onclick={onlinkbibtex}>
      Link from BibTeX…
    </button>
  {/if}
  <button
    type="button"
    role="menuitem"
    onclick={onanalyze}
    disabled={analysisState !== undefined && analysisState.phase !== "error"}
  >
    {analysisState && analysisState.phase !== "error"
      ? "Analyzing…"
      : analysisState?.phase === "error"
        ? "Retry analysis"
        : "Analyze again"}
  </button>
  <hr />
  <button
    type="button"
    role="menuitem"
    onclick={onprojectaction}
    disabled={projectActionDisabled}
  >
    {projectActionLabel}
  </button>
  <button type="button" role="menuitem" onclick={ondelete}>Delete</button>
</div>

<!-- Rendered only inside .context-menu; the button + hr chrome is defined there
     (see app.css) so both menu call sites share one style. -->
<style>
  .items {
    display: grid;
  }
</style>
