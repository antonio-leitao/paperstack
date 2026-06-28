<script lang="ts">
  // Imports the global reset + design tokens once for every window (the project
  // organizer and each standalone viewer all render under this root layout).
  import "../app.css";

  let { children } = $props();
</script>

{@render children()}
