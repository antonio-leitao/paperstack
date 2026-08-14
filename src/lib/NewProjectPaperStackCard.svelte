<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";

  let { oncreate }: { oncreate: () => void } = $props();
</script>

<button
  class="new-project-paper-stack-card"
  type="button"
  aria-label="Create a new project"
  onclick={oncreate}
>
  <span class="new-project-paper-stack-card__sheets" aria-hidden="true">
    <span class="new-project-paper-stack-card__sheet">
      <span class="new-project-paper-stack-card__content">
        <span class="new-project-paper-stack-card__icon">
          <Plus size={21} strokeWidth={1.8} />
        </span>
        <strong>New project</strong>
      </span>
    </span>
  </span>
</button>

<style>
  .new-project-paper-stack-card {
    display: block;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--ink-2);
    font: inherit;
  }

  .new-project-paper-stack-card:focus-visible {
    outline: none;
  }

  .new-project-paper-stack-card:focus-visible
    .new-project-paper-stack-card__sheets {
    border-radius: calc(var(--radius) + 3px);
    outline: 2px solid var(--accent-outline);
    outline-offset: 3px;
  }

  .new-project-paper-stack-card__sheets {
    position: relative;
    z-index: 2;
    display: grid;
    min-width: 10rem;
    aspect-ratio: 1 / 1.4;
    place-items: center;
  }

  .new-project-paper-stack-card__sheet {
    position: absolute;
    top: 0;
    display: grid;
    width: 9rem;
    aspect-ratio: 1 / 1.4;
    place-items: center;
    border: var(--bw) dashed var(--accent);
    border-radius: var(--radius);
    background: color-mix(in oklab, var(--accent) 5%, var(--card));
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
    backface-visibility: hidden;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
  }

  .new-project-paper-stack-card__content {
    display: grid;
    justify-items: center;
    gap: 8px;
    color: var(--accent-strong);
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
  }

  .new-project-paper-stack-card__icon {
    display: grid;
    color: var(--accent);
  }

  .new-project-paper-stack-card__content strong {
    font-size: var(--fs-card);
    font-weight: 600;
    line-height: 1.25;
  }

  .new-project-paper-stack-card:hover
    .new-project-paper-stack-card__sheet,
  .new-project-paper-stack-card:focus-visible
    .new-project-paper-stack-card__sheet {
    border-color: var(--accent-strong);
    background: color-mix(in oklab, var(--accent) 10%, var(--card));
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .new-project-paper-stack-card:hover .new-project-paper-stack-card__content,
  .new-project-paper-stack-card:focus-visible
    .new-project-paper-stack-card__content {
    color: var(--accent-strong);
  }

  .new-project-paper-stack-card:hover
    .new-project-paper-stack-card__icon,
  .new-project-paper-stack-card:focus-visible
    .new-project-paper-stack-card__icon {
    color: var(--accent);
  }
</style>
