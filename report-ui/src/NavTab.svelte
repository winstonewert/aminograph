<script lang="ts">
  import { location } from "svelte-spa-router";
  export let path: string;
  export let name: string;
</script>

<li class:is-active={$location.startsWith("/" + path)}>
  <a href="#/{path}/">{name}</a>
</li>
