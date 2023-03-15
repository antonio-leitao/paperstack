<script>
  import  Stack from "../components/Stack.svelte";
</script>

<h2>Home component</h2>

<div class="temp">
  <Stack />
</div>
<div class="temp">
  <Stack size={2}/>
</div>
<div class="temp">
  <Stack size={20}/>
</div>

<style>
  .temp{
    width:10rem
  }
</style>