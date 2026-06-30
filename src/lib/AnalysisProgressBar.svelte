<script lang="ts">
  import type { AnalysisStatus } from "./types";

  let {
    statuses = [],
    edge = "inline",
  }: {
    statuses?: AnalysisStatus[];
    edge?: "top" | "inline";
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
    height: 2px;
    overflow: hidden;
    pointer-events: none;
  }

  .top {
    position: absolute;
    z-index: 5;
    top: 0;
    right: 0;
    left: 0;
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
