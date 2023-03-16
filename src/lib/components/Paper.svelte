<script>
  export let title;
  function getRandomNormal(mean, stdDev) {
    let u1 = 0,
      u2 = 0;
    while (u1 === 0) u1 = Math.random();
    while (u2 === 0) u2 = Math.random();
    const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    return z0 * stdDev + mean;
  }
</script>

<h2>
  {title}
</h2>
<div class="authors">
    Mary Jane
</div>
<div class="abstract">
  Lorem ipsum dolor sit amet consectetur adipisicing elit. Aperiam tempore ullam
  nisi reiciendis ad dignissimos repellat illum, cumque unde nemo veritatis
  quasi consectetur eveniet sit aspernatur aliquid illo? Alias, itaque!
</div>

<style>
  h2 {
    font-size: 3rem;
    font-family: Arbutus Slab, serif;
    margin-top:0.5rem;
  }
</style>
