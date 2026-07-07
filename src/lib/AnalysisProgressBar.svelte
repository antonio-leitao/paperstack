<script lang="ts">
  import type { AnalysisStatus } from "./types";

  let {
    statuses = [],
    edge = "inline",
  }: {
    statuses?: AnalysisStatus[];
    edge?: "top" | "bottom" | "inline";
  } = $props();

  const resolving = $derived(
    statuses.filter(
      (status) => status.phase === "resolving" && status.total > 0,
    ),
  );
  const total = $derived(
    resolving.reduce((sum, status) => sum + status.total, 0),
  );
  const resolved = $derived(
    resolving.reduce((sum, status) => sum + status.resolved, 0),
  );
  const percent = $derived(
    total > 0 ? Math.min(100, Math.max(0, (resolved / total) * 100)) : 0,
  );
</script>

{#if resolving.length}
  <div
    class="analysis-progress"
    class:top={edge === "top"}
    class:bottom={edge === "bottom"}
    class:inline={edge === "inline"}
    role="progressbar"
    aria-label="Document analysis progress"
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={Math.round(percent)}
  >
    <span
      class="indicator"
      style:--analysis-progress={`${percent}%`}
    ></span>
  </div>
{/if}

<style>
  .analysis-progress {
    height: var(--bw-accent);
    overflow: hidden;
    /* Light-gray track; the teal indicator fills it left-to-right. */
    background: var(--progress-track);
    pointer-events: none;
  }

  /* Edge variants sit just inside the card border (padding box), so they add no
     layout height and never push the border. */
  .top,
  .bottom {
    position: absolute;
    z-index: 5;
    right: 0;
    left: 0;
  }

  .top {
    top: 0;
    border-radius: var(--radius) var(--radius) 0 0;
  }

  .bottom {
    bottom: 0;
    border-radius: 0 0 var(--radius) var(--radius);
  }

  .inline {
    position: relative;
    width: 100%;
  }

  .indicator {
    display: block;
    width: var(--analysis-progress);
    height: 100%;
    background: var(--accent);
  }
</style>
