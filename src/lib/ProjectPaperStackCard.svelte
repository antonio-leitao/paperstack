<script lang="ts">
  import type { Project } from "./types";

  let {
    project,
    onmenu,
  }: {
    project: Project;
    onmenu: (
      project: Project,
      trigger: HTMLElement,
      x: number,
      y: number,
      focusFirst: boolean,
    ) => void;
  } = $props();

  function openContextMenu(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    onmenu(
      project,
      trigger,
      event.clientX || bounds.left + 12,
      event.clientY || bounds.top + 12,
      false,
    );
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== "ContextMenu" && !(event.shiftKey && event.key === "F10")) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    onmenu(project, trigger, bounds.left + 12, bounds.top + 12, true);
  }

  function trimEllipsis(text: string): string {
    return text.length > 100 ? `${text.slice(0, 100)}...` : text;
  }

  function stackLayerCount(documentCount: number): number {
    if (documentCount <= 1) return 0;
    return Math.min(15, Math.ceil(Math.sqrt(documentCount) / 1.5));
  }

  function clampValue(value: number, limit: number): number {
    if (Math.abs(value) <= limit) return value;
    return value > 0 ? limit : -limit;
  }

  function stringToHash(value: string): number {
    let hash = 0;
    for (let index = 0; index < value.length; index += 1) {
      hash = (hash << 5) - hash + value.charCodeAt(index);
      hash |= 0;
    }
    return hash;
  }

  function getComponentSeed(title: string): number {
    const today = new Date();
    const dateSeed =
      today.getFullYear() * 10000 +
      (today.getMonth() + 1) * 100 +
      today.getDate();
    return dateSeed + stringToHash(title);
  }

  function mulberry32(seed: number): () => number {
    return () => {
      let value = (seed += 0x6d2b79f5);
      value = Math.imul(value ^ (value >>> 15), value | 1);
      value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
      return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
    };
  }

  function getTRandomNumber(df: number, random: () => number): number {
    let u = 0;
    let v = 0;
    let sum = 0;
    do {
      u = random() * 2 - 1;
      v = random() * 2 - 1;
      sum = u * u + v * v;
    } while (sum >= 1 || sum === 0);

    return clampValue(u * Math.sqrt(df * (sum / (1 - sum))), 7.5);
  }

  const paperStackVisual = $derived.by(() => {
    const layerCount = stackLayerCount(project.documentCount);
    const layerRandom = mulberry32(getComponentSeed(project.name));
    const coverRandom = mulberry32(getComponentSeed(`${project.name}:cover`));
    return {
      layers: Array.from({ length: layerCount }, () =>
        getTRandomNumber(3, layerRandom),
      ),
      coverRandom: getTRandomNumber(2, coverRandom),
      coverOrder: layerCount,
    };
  });

  const paperCount = $derived(
    `${project.documentCount} ${project.documentCount === 1 ? "paper" : "papers"}`,
  );
</script>

<a
  class="project-paper-stack-card"
  href={`/projects/${project.id}`}
  aria-label={`Open ${project.name}, ${paperCount}`}
  aria-haspopup="menu"
  oncontextmenu={openContextMenu}
  onkeydown={handleKeydown}
>
  <div class="project-paper-stack-card__sheets">
    {#each paperStackVisual.layers as random, index}
      <div
        class="project-paper-stack-card__sheet"
        style:--random={random}
        style:--order={index}
      ></div>
    {/each}

    <div
      class="project-paper-stack-card__sheet"
      style:--random={paperStackVisual.coverRandom}
      style:--order={paperStackVisual.coverOrder}
    >
      <div class="project-paper-stack-card__cover">
        <div class="project-paper-stack-card__content">
          <div class="project-paper-stack-card__title">{trimEllipsis(project.name)}</div>
          <div class="project-paper-stack-card__count">{paperCount}</div>
        </div>
      </div>
    </div>
  </div>
</a>

<style>
  .project-paper-stack-card {
    display: block;
    color: inherit;
    font: inherit;
    text-decoration: inherit;
  }

  .project-paper-stack-card:focus-visible {
    outline: none;
  }

  .project-paper-stack-card:focus-visible .project-paper-stack-card__sheets {
    border-radius: calc(var(--radius) + 3px);
    outline: 2px solid var(--accent-outline);
    outline-offset: 3px;
  }

  .project-paper-stack-card__sheets {
    position: relative;
    z-index: 2;
    display: grid;
    min-width: 10rem;
    aspect-ratio: 1 / 1.4;
    place-items: center;
  }

  .project-paper-stack-card__sheet {
    position: absolute;
    top: 0;
    display: grid;
    width: 9rem;
    aspect-ratio: 1 / 1.4;
    place-items: center;
    border-radius: var(--radius);
    background-color: var(--card);
    box-shadow:
      0 calc(var(--order) * 0.5px)
      min(calc(var(--order) * 1px), 10px)
      rgba(0, 0, 0, 0.25);
    cursor: pointer;
    transform:
      translate3d(
        calc(var(--order) * -2px),
        calc(var(--order) * -2px),
        0
      )
      rotate(calc(var(--random) * 2deg));
    transform-origin: center;
    backface-visibility: hidden;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
    will-change: transform;
  }

  .project-paper-stack-card__sheet:first-child {
    box-shadow: none;
  }

  .project-paper-stack-card:hover .project-paper-stack-card__sheet,
  .project-paper-stack-card:focus-visible .project-paper-stack-card__sheet {
    transform:
      translate3d(
        calc(var(--order) * -1px),
        calc(var(--order) * -1px),
        0
      )
      rotate(calc(var(--random) * 1deg));
  }

  .project-paper-stack-card__cover {
    position: absolute;
    display: grid;
    width: 100%;
    height: 100%;
    place-items: center;
    color: var(--ink);
  }

  .project-paper-stack-card__content {
    position: absolute;
    width: 92%;
    height: 95%;
    padding: 0.25rem;
    border-radius: 0.15rem;
  }

  .project-paper-stack-card__title {
    position: absolute;
    top: 50%;
    left: 50%;
    display: -webkit-box;
    width: 82%;
    overflow: hidden;
    color: var(--ink);
    font-family: var(--font-sans);
    font-size: var(--fs-card);
    font-weight: 600;
    line-height: 1.25;
    text-align: center;
    overflow-wrap: anywhere;
    transform: translate(-50%, -62%);
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 5;
    line-clamp: 5;
  }

  .project-paper-stack-card__count {
    position: absolute;
    right: 10%;
    bottom: 10%;
    color: var(--ink-3);
    font-family: var(--font-sans);
    font-size: var(--fs-label);
  }

</style>
