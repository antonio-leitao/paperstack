<script lang="ts">
  import { onMount } from "svelte";

  let { timestamp }: { timestamp: number } = $props();
  let now = $state(Date.now());

  onMount(() => {
    const interval = window.setInterval(() => {
      now = Date.now();
    }, 30_000);
    return () => window.clearInterval(interval);
  });

  const label = $derived(formatRelative(timestamp, now));
  const title = $derived(formatExact(timestamp));

  function formatRelative(value: number, currentTime: number): string {
    const elapsedSeconds = Math.max(0, Math.floor((currentTime - toMilliseconds(value)) / 1000));
    if (elapsedSeconds < 60) return `${Math.max(1, elapsedSeconds)}s`;
    const elapsedMinutes = Math.floor(elapsedSeconds / 60);
    if (elapsedMinutes < 60) return `${elapsedMinutes}m`;
    const elapsedHours = Math.floor(elapsedMinutes / 60);
    if (elapsedHours < 24) return `${elapsedHours}h`;
    const elapsedDays = Math.floor(elapsedHours / 24);
    if (elapsedDays < 7) return `${elapsedDays}d`;
    const elapsedWeeks = Math.floor(elapsedDays / 7);
    if (elapsedWeeks < 52) return `${elapsedWeeks}w`;
    return `${Math.floor(elapsedWeeks / 52)}y`;
  }

  function formatExact(value: number): string {
    return new Date(toMilliseconds(value)).toLocaleString();
  }

  function toMilliseconds(value: number): number {
    return value > 1_000_000_000_000 ? value : value * 1000;
  }
</script>

<time datetime={new Date(toMilliseconds(timestamp)).toISOString()} {title}>{label}</time>

<style>
  time {
    color: var(--ink-3);
    font-size: var(--font-size-small);
    white-space: nowrap;
  }
</style>
