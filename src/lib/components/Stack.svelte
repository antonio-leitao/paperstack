<script>
  import { link } from "svelte-spa-router";
  function randomFloat(start, end) {
    return Math.random() * (end - start) + start;
  }
  export let size = 7;
  function getRandomNormal(mean, stdDev) {
    let u1 = 0,
      u2 = 0;
    while (u1 === 0) u1 = Math.random();
    while (u2 === 0) u2 = Math.random();
    const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    return z0 * stdDev + mean;
  }
</script>

<a href={"/publications"} use:link>
  <div class="stack">
    <div class="label">PUBLICATIONS</div>
    {#each { length: size } as _, i}
      <div class="paper" style="--random:{getRandomNormal(0, 2)}; --order:{i}" />
    {/each}
  </div>
</a>

<style>
  .label {
    font-family: "Helvetica", "Arial", sans-serif;
    position: absolute;
    top: 30%;
    color: var(--text1);
    font-size: 0.8rem;
    font-weight: 100;
    text-transform: uppercase;
    letter-spacing: 0.2rem;
    margin-bottom: 0.5rem;
    z-index: 5;
    background-color: rgb(0, 0, 0, 0.8);
    background-color: var(--clr-mixred-dark);
    padding: 0.5rem;
    cursor: pointer;
    text-shadow: 0 2px 10px rgba(0, 0, 0, 1);
  }
  .stack {
    display: grid;
    place-items: center;
    margin: 2rem;
    position: relative;
    min-width: 7rem;
    aspect-ratio: 1/1.4;
  }
  .paper {
    position: absolute;
    top: 0;
    display: grid;
    place-items: center;
    border: 1px solid rgba(0, 0, 0, 0.3);
    border-radius: 5px;
    width: 5rem;
    aspect-ratio: 1/1.4;
    cursor: pointer;
    /* Functional props*/
    position: absolute;
    --rotation: calc(calc(30deg / -4) + calc(calc(30deg / 25)) * var(--order));
    --y_offset: calc(var(--order) * -2px);
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--y_offset)));
    transform-origin: center;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
    background-color: white;
    box-shadow: 1px 1px 2px rgba(0, 0, 0, 0.4);
  }

  .stack:hover > .paper {
    transform: rotate(calc(calc(60deg / -2) + 10deg * var(--order)));
    transform-origin: center 120%;
    box-shadow: -15px 2px 15px rgba(0, 0, 0, 0.07),
      0px 0px 8px rgba(0, 0, 0, 0.07);
  }
  a {
    color: inherit;
    font: inherit;
    text-decoration: inherit;
  }
</style>
