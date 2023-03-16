<script>
  import Paper from "../components/Paper.svelte";
  export let num;
  export let params = {};
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
  let papers = [
    {
      title: "This is the name of a paper",
    },
    {
      title: "This is the name of another paper.",
      subtitle: "第二張測試卡片",
    },
    {
      title: "Yet another but with some entonation",
      subtitle: "第三張測試卡片",
    },
    {
      title: "Attention - this is the penultimate paper.",
      subtitle: "第四張測試卡片",
    },
    {
      title: "Last paper and the results that come within: a survey.",
      subtitle: "最後一張測試卡片",
    },
  ];
  let N = papers.length + 1;
</script>

<h2>Collection Name</h2>

<p>
  route props:
  <b>{params.first}</b>
  <b
    >{#if params.last}{params.last}{/if}</b
  >
</p>

<p>
  static props: {num}
</p>

<div class="cards_wrap">
  {#each papers as paper, i}
    {#if i == 0}
      <div
        class="cards_wrap__card"
        style="--random:{getTRandomNumber(2)};--order:{i+1};height: {90 *
          (N - i)}vh;"
      >
        <div class="cards_wrap__card-stick">
          <Paper title={paper.title} />
        </div>
      </div>
    {:else}
      <div
        class="cards_wrap__card"
        style="--random:{getTRandomNumber(3)};--order:{i+1};height: {90 *
          (N - i)}vh;margin-top: {90 * (i - N)}vh;"
      >
        <div class="cards_wrap__card-stick">
          <Paper title={paper.title} />
        </div>
      </div>
    {/if}
  {/each}
</div>
<div class="footer">
  This is Footer&nbsp;<span>
    Made by <a target="_blank" href="https://e-s.tw/">ES Design - Eason</a
    ></span
  >
</div>

<style>
  .footer {
    position: relative;
    z-index: 1;
    height: 50vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .footer span {
    display: block;
  }

  .cards_wrap__card-stick {
    top: 2rem;
    position: sticky;
    display: flex;
    align-items: left;
    justify-content: center;
    flex-direction: column;
    border-radius: 2rem;
    background-position: center;
    background-color: white;
    /* border: 1px lightgray solid; */
    height: 90vh;
    padding: 2rem;
    --shadow: calc(var(--order) * 3px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 50px)
      rgba(0, 0, 0, 0.25);
    transform: rotate(calc(var(--random) * 2deg))
    translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
  }
</style>
