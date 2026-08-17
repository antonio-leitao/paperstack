<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Check from "@lucide/svelte/icons/check";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import X from "@lucide/svelte/icons/x";
  import type { ProviderSettingStatus, ProviderSettingsStatus } from "$lib/types";
  import { errorMessage } from "$lib/errorMessage";
  import Modal from "$lib/Modal.svelte";

  let {
    open,
    onclose,
  }: {
    open: boolean;
    onclose: () => void;
  } = $props();

  let settings = $state<ProviderSettingsStatus | null>(null);
  let openalexApiKey = $state("");
  let semanticScholarApiKey = $state("");
  let crossrefMailto = $state("");
  let clearOpenalex = $state(false);
  let clearSemanticScholar = $state(false);
  let clearCrossref = $state(false);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  function handleOpen() {
    openalexApiKey = "";
    semanticScholarApiKey = "";
    crossrefMailto = "";
    clearOpenalex = false;
    clearSemanticScholar = false;
    clearCrossref = false;
    error = null;
    void loadSettings();
  }

  async function loadSettings() {
    loading = true;
    try {
      settings = await invoke<ProviderSettingsStatus>("get_provider_settings");
      error = null;
    } catch (caught) {
      error = errorMessage(caught);
    } finally {
      loading = false;
    }
  }

  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    try {
      settings = await invoke<ProviderSettingsStatus>("save_provider_settings", {
        settings: {
          openalexApiKey: openalexApiKey.trim() || null,
          semanticScholarApiKey: semanticScholarApiKey.trim() || null,
          crossrefMailto: crossrefMailto.trim() || null,
          clearOpenalexApiKey: clearOpenalex,
          clearSemanticScholarApiKey: clearSemanticScholar,
          clearCrossrefMailto: clearCrossref,
        },
      });
      error = null;
      onclose();
    } catch (caught) {
      error = errorMessage(caught);
    } finally {
      saving = false;
    }
  }

  function sourceLabel(status: ProviderSettingStatus | undefined, clearing: boolean) {
    if (!status) return "Checking configuration…";
    if (clearing) {
      return status.hasEnvironmentValue
        ? `Will use ${status.environmentVariable} after saving`
        : "Will use anonymous access after saving";
    }
    if (status.hasOverride) return "Saved override active";
    if (status.hasEnvironmentValue) return `${status.environmentVariable} detected`;
    return "No configured value — anonymous access";
  }

  function fieldPlaceholder(status: ProviderSettingStatus | undefined) {
    if (!status) return "Checking configuration…";
    if (status.hasOverride) return "Saved override — enter a new value to replace it";
    if (status.hasEnvironmentValue) return `Using ${status.environmentVariable}`;
    return "Optional override";
  }
</script>

<Modal
  {open}
  labelledby="provider-settings-title"
  size="wide"
  onopen={handleOpen}
  onclose={onclose}
>
  <form class="dialog-form settings-form" onsubmit={save}>
    <div class="dialog-heading">
      <div>
        <h2 id="provider-settings-title">Settings</h2>
        <p>
          Saved overrides stay on this device and take precedence over environment values.
          If an API key is rejected, PaperStack tries the environment key, then anonymous access.
          Environment values are detected but never displayed.
        </p>
      </div>
      <button class="quiet-btn quiet-btn--icon" type="button" aria-label="Close settings" onclick={onclose}>
        <X size={17} strokeWidth={1.8} aria-hidden="true" />
      </button>
    </div>

    <fieldset disabled={loading || saving}>
      <legend>Metadata providers</legend>

      <div class="provider-setting">
        <div class="setting-heading">
          <label for="openalex-key">OpenAlex API key</label>
          <span class="source-status">{sourceLabel(settings?.openalex, clearOpenalex)}</span>
        </div>
        <p>
          Raises request quotas while resolving papers, abstracts, authors, and open-access links.
        </p>
        <div class="input-row">
          <input
            id="openalex-key"
            type="password"
            autocomplete="off"
            bind:value={openalexApiKey}
            placeholder={fieldPlaceholder(settings?.openalex)}
            oninput={() => (clearOpenalex = false)}
          />
          {#if settings?.openalex.hasOverride}
            <button class="paper-btn" type="button" onclick={() => (clearOpenalex = !clearOpenalex)}>
              <RotateCcw size={14} strokeWidth={1.8} aria-hidden="true" />
              <span>{clearOpenalex ? "Keep override" : "Use fallback"}</span>
            </button>
          {/if}
        </div>
      </div>

      <div class="provider-setting">
        <div class="setting-heading">
          <label for="semantic-key">Semantic Scholar API key</label>
          <span class="source-status">{sourceLabel(settings?.semanticScholar, clearSemanticScholar)}</span>
        </div>
        <p>
          Authenticates final-stage metadata searches for unresolved references.
        </p>
        <div class="input-row">
          <input
            id="semantic-key"
            type="password"
            autocomplete="off"
            bind:value={semanticScholarApiKey}
            placeholder={fieldPlaceholder(settings?.semanticScholar)}
            oninput={() => (clearSemanticScholar = false)}
          />
          {#if settings?.semanticScholar.hasOverride}
            <button class="paper-btn" type="button" onclick={() => (clearSemanticScholar = !clearSemanticScholar)}>
              <RotateCcw size={14} strokeWidth={1.8} aria-hidden="true" />
              <span>{clearSemanticScholar ? "Keep override" : "Use fallback"}</span>
            </button>
          {/if}
        </div>
      </div>

      <div class="provider-setting">
        <div class="setting-heading">
          <label for="crossref-email">Crossref contact email</label>
          <span class="source-status">{sourceLabel(settings?.crossref, clearCrossref)}</span>
        </div>
        <p>
          Adds a contact address to Crossref requests, enabling its polite pool for faster, more
          reliable DOI and bibliography lookups. It is not an API key.
        </p>
        <div class="input-row">
          <input
            id="crossref-email"
            type="email"
            autocomplete="email"
            bind:value={crossrefMailto}
            placeholder={fieldPlaceholder(settings?.crossref)}
            oninput={() => (clearCrossref = false)}
          />
          {#if settings?.crossref.hasOverride}
            <button class="paper-btn" type="button" onclick={() => (clearCrossref = !clearCrossref)}>
              <RotateCcw size={14} strokeWidth={1.8} aria-hidden="true" />
              <span>{clearCrossref ? "Keep override" : "Use fallback"}</span>
            </button>
          {/if}
        </div>
      </div>
    </fieldset>

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    <div class="dialog-actions">
      <button class="paper-btn" type="button" disabled={saving} onclick={onclose}>
        Cancel
      </button>
      <button class="paper-btn paper-btn--primary" type="submit" disabled={loading || saving}>
        <Check size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>{saving ? "Saving…" : "Save settings"}</span>
      </button>
    </div>
  </form>
</Modal>

<style>
  .settings-form {
    gap: 16px;
  }

  .dialog-heading,
  .setting-heading,
  .input-row {
    display: flex;
    align-items: center;
  }

  .dialog-heading {
    align-items: flex-start;
    justify-content: space-between;
    gap: 18px;
  }

  h2,
  p {
    margin: 0;
  }

  .dialog-heading p {
    max-width: 500px;
    margin-top: 5px;
    color: var(--ink-2);
    line-height: 1.45;
  }

  fieldset {
    display: grid;
    gap: 0;
    margin: 0;
    padding: 0 14px;
    border-radius: var(--radius);
  }

  legend {
    padding: 0 6px;
    color: var(--ink-3);
    font-weight: 600;
  }

  .provider-setting {
    display: grid;
    gap: 7px;
    padding: 15px 0;
    border-bottom: var(--bw) solid var(--line);
  }

  .provider-setting:last-child {
    border-bottom: 0;
  }

  .setting-heading {
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 5px 12px;
  }

  label {
    font-weight: 600;
  }

  .source-status {
    color: var(--ink-3);
    font-size: var(--fs-meta);
  }

  .provider-setting p {
    color: var(--ink-2);
    line-height: 1.45;
  }

  .input-row {
    align-items: stretch;
    gap: 6px;
  }

  input {
    min-width: 0;
    flex: 1;
    padding: 7px 9px;
  }

  .error {
    color: var(--danger);
  }

  @media (max-width: 560px) {
    .input-row {
      flex-direction: column;
    }

    .input-row .paper-btn {
      align-self: flex-start;
    }
  }
</style>
