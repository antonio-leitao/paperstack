<script>
  import { link } from "svelte-spa-router";

  function trimElipsis(text) {
    if (text.length > 60) {
      text = text.slice(0, 100) + "...";
    }
    return text;
  }
  export let title = "Topological effectivenes of machine learning paradigms";
  export let stackId;
  export let size = 7;
  export let image = "";
  export let description = "";
  export let locked = false;
  let hasImage = false;
  if (image) {
    hasImage = true;
  }
  function trimSize(size) {
    if (size > 8) {
      size = Math.floor(3 * Math.log2(size));
    }
    return size;
  }

  function getRandomNormal(mean, stdDev) {
    let u1 = 0,
      u2 = 0;
    while (u1 === 0) u1 = Math.random();
    while (u2 === 0) u2 = Math.random();
    const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    return z0 * stdDev + mean;
  }
  function getTRandomNumber(df) {
    var z1 = Math.random() * 2 - 1; // uniform(-1, 1)
    var z2 = Math.random() * 2 - 1; // uniform(-1, 1)
    var s = z1 * z1 + z2 * z2;
    while (s === 0 || s > 1) {
      z1 = Math.random() * 2 - 1; // uniform(-1, 1)
      z2 = Math.random() * 2 - 1; // uniform(-1, 1)
      s = z1 * z1 + z2 * z2;
    }
    var t = z1 * Math.sqrt(df * (s / (1 - s)));

    return t;
  }
</script>

<a href={"/stacks/" + stackId} use:link>
  <div class="stack">
    {#each { length: trimSize(size) } as _, i}
      <div class="paper" style="--random:{getTRandomNumber(4)}; --order:{i}" />
    {/each}
    <div
      class="paper"
      class:image={hasImage}
      style="--image:url({image});--random:{getTRandomNumber(
        2
      )}; --order:{trimSize(size)};"
    >
      <div class="cover">
        <div class="content" class:noborder={hasImage}>
          <div class="title">{trimElipsis(title)}</div>
          <div class="count">{size} {size == 1 ? "paper" : "papers"}</div>
        </div>
      </div>
      <div />
    </div>
  </div></a
>

<style>
  .paper:first-child {
    box-shadow: none;
  }
  .stack:hover > .paper {
    transform: rotate(calc(var(--random) * 1deg))
      translate(calc(var(--order) * -1px), calc(var(--order) * -1px));
    box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
      rgba(0, 0, 0, 0.25);
  }
  a {
    color: inherit;
    font: inherit;
    text-decoration: inherit;
  }
  .cover {
    position: absolute;
    color: var(--text1);
    width: 100%;
    height: 100%;
    display: grid;
    place-items: center;
  }
  .image {
    background-image: var(--image);
    background-position: center;
    background-size: cover;
  }
  .content {
    position: absolute;
    width: 92%;
    height: 95%;
    border-radius: 0.15rem;
    border: 1px lightgray solid;
    padding: 0.25rem;
  }
  .noborder {
    border: none;
  }
  .title {
    font-family: "Lora", serif;
    font-weight: 500;
    font-size: 0.95rem;
    margin-top: 0.5rem;
  }
  .count {
    font-size: 0.6rem;
    color: lightgrey;
    position: absolute;
    bottom: 10%;
    right: 10%;
  }
</style>
