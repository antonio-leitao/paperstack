<script>
  export let notes;
  function getRandomNormal(mean, stdDev) {
    let u1 = 0,
      u2 = 0;
    while (u1 === 0) u1 = Math.random();
    while (u2 === 0) u2 = Math.random();
    const z0 = Math.sqrt(-2.0 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    return z0 * stdDev + mean;
  }
</script>

<div class="postit --random:{getRandomNormal(0, 2)}">
  <div class="note">
    {notes}
  </div>
</div>

<style>
  .postit {
    position: absolute;
    top: 5%;
    right: 5%;
    padding: 0.7rem;
    font-weight: 100;
    font-size: 0.7rem;
    background-color: #ffffa5;
    width: 25%;
    aspect-ratio: 1/1;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.27),
      0 0 40px rgba(0, 0, 0, 0.06) inset;
    transform: rotate(calc(var(--random) * 4deg))
      translate(calc(var(--random) * -20px), calc(var(--random) * -10px));
  }
</style>
