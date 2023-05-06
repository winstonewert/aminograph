<script lang="ts">
  import REPORT, { ReportMetadata } from "./report";
  import NavTab from "./NavTab.svelte";
  import { AMINO_ACIDS } from "./types";
  export let node_id: string;
  $: node = REPORT[node_id];
  const SIZE = 100;
</script>

<h1 class="is-size-1">
  {#if node.sequence_id}
    {#if ReportMetadata.sequences[node.sequence_id]}
      {#if ReportMetadata.sequences[node.sequence_id].image}
        <img
          src={ReportMetadata.sequences[node.sequence_id].image}
          class="sequence-image"
          alt=""
        />
      {/if}
      <div class="sequence-label">
        {ReportMetadata.sequences[node.sequence_id].label} ({ReportMetadata
          .sequences[node.sequence_id].sublabel})
      </div>
    {:else}
      {node.sequence_id}
    {/if}
  {:else}
    {node_id}
  {/if}
</h1>
<div class="layout">
  <div class="tabs">
    <ul>
      <NavTab path="nodes/{node_id}" name="Overview" />
      <NavTab path="nodes/{node_id}/sequence" name="Sequence" />
      <NavTab path="nodes/{node_id}/graph" name="Graph" />
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

  .sequence-image {
    width: 64px;
    height: 64px;
    display: inline-block;
    margin-left: 0.5em;
  }

  .sequence-label {
    display: inline-block;
    vertical-align: top;
  }
</style>
