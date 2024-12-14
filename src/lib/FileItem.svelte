<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  let {
    id,
    url,
    image,
    pdf,
    bib,
    pages,
    compactView = false,
    ondrop,
    ondragenter,
    ondragover,
    ondragleave,
    isSelected = false,
    isBeingDraggedOver = false,
  } = $props();
  let year = bib?.issued
    ? bib.issued["date-parts"]?.[0]?.[0] || "Unknown year"
    : "Unknown year";
  let author = bib?.author
    ? bib.author.map((author) =>
        `${author.given || ""} ${author.family || ""}`.trim()
      )
    : [];
  let image_url = $derived(image ? convertFileSrc(image) : null);
  let hasImage = $derived(image_url !== null);
  let page_count = $derived(trimSize(pages));

  function trimSize(x) {
    return Math.floor(Math.max(1, Math.min(15, Math.sqrt(x) / 1.5)));
  }

  function clampValue(value, limit) {
    if (Math.abs(value) > limit) {
      return value > 0 ? limit : -limit;
    }
    return value;
  }

  function stringToHash(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash |= 0; // Convert to 32bit integer
    }
    return hash;
  }

  function getComponentSeed(title) {
    const today = new Date();
    const dateSeed =
      today.getFullYear() * 10000 +
      (today.getMonth() + 1) * 100 +
      today.getDate();
    const titleHash = stringToHash(title);
    return dateSeed + titleHash; // Combine date and title hash
  }

  // Linear Congruential Generator (LCG) for pseudo-random numbers from a seed
  function seededRandom(seed) {
    const m = 2 ** 32;
    const a = 1664525;
    const c = 1013904223;

    let z = seed;
    return function () {
      z = (a * z + c) % m;
      return z / m;
    };
  }

  // Create random number generator with today's seed
  const componentSeed = getComponentSeed(bib.title);
  const random = seededRandom(componentSeed);

  function getTRandomNumber(df) {
    let z1 = random() * 2 - 1; // uniform(-1, 1)
    let z2 = random() * 2 - 1; // uniform(-1, 1)
    let s = z1 * z1 + z2 * z2;
    while (s === 0 || s > 1) {
      z1 = random() * 2 - 1; // uniform(-1, 1)
      z2 = random() * 2 - 1; // uniform(-1, 1)
      s = z1 * z1 + z2 * z2;
    }
    let t = z1 * Math.sqrt(df * (s / (1 - s)));

    t = clampValue(t, 7.5);
    return t;
  }
  $effect(() => {
    console.log(pages);
  });
</script>

{#if compactView}
  <div class="stack compact" {ondrop} {ondragenter} {ondragover} {ondragleave}>
    <h3>{bib.title}</h3>
    <p class="subinfo">{year}</p>
    <p class="subinfo">
      {author.join(", ")}
    </p>
  </div>
{:else}
  <div class="stack" {ondrop} {ondragenter} {ondragover} {ondragleave}>
    {#each { length: page_count } as _, i}
      <div
        class="paper"
        style="--random:{getTRandomNumber(3)}; --order:{i}"
      ></div>
    {/each}

    <div
      class="paper"
      style="--random:{getTRandomNumber(2)}; --order:{page_count};"
    >
      <div
        class="cover"
        class:selected={isSelected}
        class:dragover={isBeingDraggedOver}
      >
        {#if image}
          <img src={image_url} alt={bib.title} class="background-image" />
        {/if}
        <div class="overlay" class:image-present={hasImage}>
          <div class="text-content" class:top-positioned={!hasImage}>
            <p class="subinfo">{year}</p>
            <h3>{bib.title}</h3>
            <p class="subinfo">
              {author.join(", ")}
            </p>
          </div>
        </div>
      </div>
      <div></div>
    </div>
  </div>
{/if}

<style>
  .stack {
    display: grid;
    place-items: center;
    position: relative;
    min-width: 10rem;
    aspect-ratio: 1/1.4;
  }
  .stack.compact {
    height: 3rem;
    width: 100%;
  }
  .paper {
    position: absolute;
    top: 0;
    display: grid;
    place-items: center;
    /* border: 1px solid rgba(0, 0, 0, 0.2); */
    border-radius: 5px;
    width: 12rem;
    aspect-ratio: 1/1.4;
    cursor: pointer;
    /* Functional props*/
    --rotation: calc(calc(30deg / -4) + calc(calc(30deg / 25)) * var(--order));
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
    transform-origin: center;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
    background-color: white;
    --shadow: calc(var(--order) * 1px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
      rgba(0, 0, 0, 0.25);
    box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
      rgba(0, 0, 0, 0.25);
  }
  .paper:first-child {
    box-shadow: none;
  }
  .stack:hover > .paper {
    transform: rotate(calc(var(--random) * 1deg))
      translate(calc(var(--order) * -1px), calc(var(--order) * -1px));
    box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
      rgba(0, 0, 0, 0.25);
  }
  .cover {
    position: absolute;
    color: var(--hover);
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    border-radius: 5px;
    overflow-y: hidden;
  }
  .overlay {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .overlay.image-present {
    justify-content: flex-end;
  }
  .text-content {
    padding: 0.5rem;
    background-color: white;
    z-index: 2;
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;
  }
  .text-content.top-positioned {
    margin-top: 1rem;
  }
  /* Image styling */
  .background-image {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    border-top-left-radius: 5px;
    border-top-right-radius: 5px;
    object-fit: cover;
    z-index: -1;
  }

  .selected {
    border: 2px solid var(--accent-color-light);
  }
  .dragover {
    border: 2px solid var(--secondary-color-light);
  }
</style>
