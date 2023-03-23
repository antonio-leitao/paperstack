<script>
  import Stack from "./Stack.svelte";
  let image =
    "https://media.springernature.com/w440/springer-static/cover-hires/journal/41586/615/7952";
  let title = "Twist Factor";
  let publisher = "Nature";
  let description =
    " Lorem ipsum dolor sit amet consectetur adipisicing elit. Delectus exercitationem omnis ut odio, illo aut iusto. Fugit sed voluptas unde impedit ex iusto nam voluptates consequatur nemo similique, consectetur aut.";
</script>

<div class="backprop content">
  <div class="left">
    <Stack {image} stackId={321321} title={""} size={10} />
  </div>
  <div class="right">
    <h3>{publisher}</h3>
    <h1>{title}</h1>
    <div class="description">
      {description}
    </div>
  </div>
</div>

<style>
  .backprop {
    position: relative;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    overflow: hidden;
    border: 0.5vmin solid var(--main-red);
    position: relative;
  }
  .right {
    flex-direction: column;
    justify-content: flex-start;
    height: 100%;
  }
  h3 {
    text-transform: uppercase;
    font-weight:100;
  }
  .content {
    background-image: radial-gradient(rgba(0, 0, 0, 0.2) 8%, transparent 8%);
    background-position: 0% 0%;
    background-size: 5vmin 5vmin;
    height: calc(100% - 10vmin);
    padding: 5vmin;
    position: relative;
    transition: background-position 350ms ease;
    width: calc(100% - 10vmin);
    z-index: 2;
  }
</style>
