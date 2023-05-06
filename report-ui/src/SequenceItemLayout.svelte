<script lang="ts">
  import REPORT from "./report";
  import NavTab from "./NavTab.svelte";
  import { AMINO_ACIDS } from "./types";
  export let index: number;
</script>

<h1 class="is-size-1">
  {index+1}
</h1>
<div class="layout">
  <div class="tabs">
    <ul>
      <NavTab path="sequence/{index}" name="Overview" />
      <NavTab path="sequence/{index}/graph" name="Graph" />
    </ul>
  </div>
  <div class="contain">
    <slot />
  </div>
</div>

<style>
  .contain {
    overflow-y: auto;
    width: 100%;
    flex-grow: 1;
  }
  .tabs {
    width: 160px;
  }
  .tabs ul {
    flex-direction: column;
  }
  .tabs li + li {
    margin-left: 0;
  }
  .tabs li {
    width: 100%;
  }
  .nn {
    overflow-y: auto;
  }
  table {
    margin-bottom: 1rem;
  }
  .residue {
    width: 0.75rem;
  }
  .residue.differs {
    color: red;
  }
  .source {
    width: 5rem;
    font-weight: bold;
  }
  .layout {
    display: flex;
    overflow-y: auto;
    height: 100%;
  }
</style>
