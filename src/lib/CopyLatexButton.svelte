<script lang="ts">
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
  {status === "copying"
    ? "Copying…"
    : status === "copied"
      ? "Copied"
      : status === "empty"
        ? "No highlights"
        : status === "failed"
          ? failureMessage
          : "Copy as LaTeX"}
</button>
