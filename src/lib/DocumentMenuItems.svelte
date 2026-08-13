<script lang="ts">
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FolderMinus from "@lucide/svelte/icons/folder-minus";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import FolderPlus from "@lucide/svelte/icons/folder-plus";
  import Link2 from "@lucide/svelte/icons/link-2";
  import MessageSquareText from "@lucide/svelte/icons/message-square-text";
  import MessageSquareX from "@lucide/svelte/icons/message-square-x";
  import Pencil from "@lucide/svelte/icons/pencil";
  import ScanSearch from "@lucide/svelte/icons/scan-search";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import Unlink from "@lucide/svelte/icons/unlink";
  import CopyBibtexButton from "./CopyBibtexButton.svelte";
  import CopyCitationKeyButton from "./CopyCitationKeyButton.svelte";
  import CopyLatexButton from "./CopyLatexButton.svelte";
  import type { AnalysisStatus, LibraryDocument } from "./types";

  let {
    document,
    analysisState,
    projectActionLabel,
    projectActionDisabled = false,
    onopen,
    onshowinfolder,
    onrename,
    oneditnote,
    onremovenote,
    onlinkbibtex,
    onunlink,
    onanalyze,
    oncopylatex,
    onprojectaction,
    ondelete,
  }: {
    document: LibraryDocument;
    analysisState?: AnalysisStatus;
    projectActionLabel: string;
    projectActionDisabled?: boolean;
    onopen: () => void;
    onshowinfolder: () => void;
    onrename: () => void;
    oneditnote: () => void;
    onremovenote: () => void;
    onlinkbibtex: () => void;
    onunlink: () => void;
    onanalyze: () => void;
    oncopylatex?: () => Promise<boolean>;
    onprojectaction: () => void;
    ondelete: () => void;
  } = $props();
</script>

<div class="items">
  <div class="menu-group" role="group">
    <button type="button" role="menuitem" onclick={onopen}>
      <ExternalLink size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>Open</span>
    </button>
    <button type="button" role="menuitem" onclick={onshowinfolder}>
      <FolderOpen size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>Show in Folder</span>
    </button>
  </div>

  {#if document.referenceBibtex || oncopylatex}
    <hr />
    <div class="menu-group" role="group">
      {#if document.referenceBibtex}
        <CopyBibtexButton bibtex={document.referenceBibtex} menuItem />
        <CopyCitationKeyButton bibtex={document.referenceBibtex} menuItem />
      {/if}
      {#if oncopylatex}
        <CopyLatexButton oncopy={oncopylatex} />
      {/if}
    </div>
  {/if}

  <hr />
  <div class="menu-group" role="group">
    <button type="button" role="menuitem" onclick={onrename}>
      <Pencil size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>Rename</span>
    </button>
    <button type="button" role="menuitem" onclick={oneditnote}>
      <MessageSquareText size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>{document.note ? "Edit note…" : "Add note…"}</span>
    </button>
    {#if document.note}
      <button
        class="context-menu-danger"
        type="button"
        role="menuitem"
        onclick={onremovenote}
      >
        <MessageSquareX size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Remove note</span>
      </button>
    {/if}
    {#if document.referenceId}
      <button
        class="context-menu-danger"
        type="button"
        role="menuitem"
        onclick={onunlink}
      >
        <Unlink size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Unlink reference</span>
      </button>
    {:else}
      <button type="button" role="menuitem" onclick={onlinkbibtex}>
        <Link2 size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Link from BibTeX…</span>
      </button>
    {/if}
    <button
      type="button"
      role="menuitem"
      onclick={onanalyze}
      disabled={analysisState !== undefined && analysisState.phase !== "error"}
    >
      <ScanSearch size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>
        {analysisState && analysisState.phase !== "error"
          ? "Analyzing…"
          : analysisState?.phase === "error"
            ? "Retry analysis"
            : "Analyze again"}
      </span>
    </button>
  </div>

  <hr />
  <div class="menu-group" role="group">
    <button
      class:context-menu-danger={projectActionLabel.startsWith("Remove")}
      type="button"
      role="menuitem"
      onclick={onprojectaction}
      disabled={projectActionDisabled}
    >
      {#if projectActionLabel.startsWith("Remove")}
        <FolderMinus size={16} strokeWidth={1.8} aria-hidden="true" />
      {:else}
        <FolderPlus size={16} strokeWidth={1.8} aria-hidden="true" />
      {/if}
      <span>{projectActionLabel}</span>
    </button>
  </div>

  <hr />
  <div class="menu-group" role="group">
    <button
      class="context-menu-danger"
      type="button"
      role="menuitem"
      onclick={ondelete}
    >
      <Trash2 size={16} strokeWidth={1.8} aria-hidden="true" />
      <span>Delete</span>
    </button>
  </div>
</div>

<!-- Rendered only inside .context-menu; the button + hr chrome is defined there
     (see app.css) so both menu call sites share one style. -->
<style>
  .items {
    display: grid;
  }
</style>
