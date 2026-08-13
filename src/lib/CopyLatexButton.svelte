<script lang="ts">
  import Braces from "@lucide/svelte/icons/braces";
  import Check from "@lucide/svelte/icons/check";
  import CircleSlash from "@lucide/svelte/icons/circle-slash";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import TriangleAlert from "@lucide/svelte/icons/triangle-alert";

  let {
    oncopy,
  }: {
    oncopy: () => Promise<boolean>;
  } = $props();

  let status = $state<"idle" | "copying" | "copied" | "empty" | "failed">("idle");
  let failureMessage = $state("Copy failed");
  let resetTimer: number | null = null;

  async function copy(event: MouseEvent) {
    event.stopPropagation();
    if (status === "copying") return;
    if (resetTimer !== null) window.clearTimeout(resetTimer);
    status = "copying";
    try {
      status = (await oncopy()) ? "copied" : "empty";
    } catch (error) {
      failureMessage =
        error instanceof Error && error.message.trim()
          ? error.message
          : typeof error === "string" && error.trim()
            ? error
          : "Copy failed";
      status = "failed";
    }
    resetTimer = window.setTimeout(() => {
      status = "idle";
      resetTimer = null;
    }, 1800);
  }
</script>

<button type="button" role="menuitem" onclick={copy} disabled={status === "copying"}>
  {#if status === "copying"}
    <LoaderCircle class="menu-icon--spin" size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "copied"}
    <Check size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "empty"}
    <CircleSlash size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else if status === "failed"}
    <TriangleAlert size={16} strokeWidth={1.8} aria-hidden="true" />
  {:else}
    <Braces size={16} strokeWidth={1.8} aria-hidden="true" />
  {/if}
  <span>
    {status === "copying"
      ? "Copying…"
      : status === "copied"
        ? "Copied"
        : status === "empty"
          ? "No highlights"
          : status === "failed"
            ? failureMessage
            : "Copy as LaTeX"}
  </span>
</button>
