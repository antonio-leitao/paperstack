<script lang="ts">
  import "./styles.css";

  let { children } = $props();
</script>

<main>
  {@render children()}
</main>
<style>
main{
    margin-top:2rem;
}
</style>

