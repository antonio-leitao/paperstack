<script>
  import { link } from "svelte-spa-router";

  export let title = "Topological effectiveness of machine learning paradigms";
  export let size = 7;
  if (size > 8) {
    size = Math.floor(3 * Math.log2(size));
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
    // df: degrees of freedom
    // returns a random number from a t-distribution with the given degrees of freedom

    // generate two independent standard normal random variables
    var z1 = Math.random() * 2 - 1; // uniform(-1, 1)
    var z2 = Math.random() * 2 - 1; // uniform(-1, 1)

    // calculate the squared norm of the vector (z1, z2)
    var s = z1 * z1 + z2 * z2;

    // if s is zero, generate a new pair of random variables
    while (s === 0 || s > 1) {
      z1 = Math.random() * 2 - 1; // uniform(-1, 1)
      z2 = Math.random() * 2 - 1; // uniform(-1, 1)
      s = z1 * z1 + z2 * z2;
    }

    // calculate the t-distributed random variable
    var t = z1 * Math.sqrt(df * (s / (1 - s)));

    return t;
  }
</script>

<a href={"/publications"} use:link>
  <div class="stack">
    {#each { length: size } as _, i}
      <div class="paper" style="--random:{getTRandomNumber(5)}; --order:{i}" />
    {/each}
    <div class="paper" style="--random:{getTRandomNumber(3)}; --order:{size}">
      <div class="cover">
        <div class="content">
          <div class="title">{title}</div>
          <div class="count">{size} papers</div>
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
  /* .stack:hover > .paper{
    transform: rotate(0) translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
  } */
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
    display:grid;
    place-items: center;
  }
  .content {
    position: absolute;
    width: 92%;
    height:95%;
    border-radius: 0.15rem;
    border: 1px lightgray solid;
    padding: 0.25rem;
    
  }
  .title{
    font-family: Arbutus Slab, serif;
    font-size:0.95rem;
    margin-top:0.5rem;
  }
  .count{
    font-family: Open 'Lucida Sans', 'Lucida Sans Regular', 'Lucida Grande', 'Lucida Sans Unicode', Geneva, Verdana, sans-serif;
    font-size:0.6rem;
    color:lightgrey;
    position:absolute;
    bottom:10%;
    right:10%;
  }
</style>
