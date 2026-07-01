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

<style>
  .items {
    display: grid;
  }

  button {
    padding: 4px 8px;
    text-align: left;
    white-space: nowrap;
  }

  hr {
    width: 100%;
    margin: 4px 0;
    border: 0;
    border-top: 1px solid var(--border-subtle);
  }
</style>
