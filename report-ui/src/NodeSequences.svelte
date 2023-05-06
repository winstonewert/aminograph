<script lang="ts">
  import REPORT, {
    allDependencies,
    changeMode,
    interestingPositions,
    nodeDepends,
    node_position_inherited,
    topologicalOrder,
    ReportMetadata,
  } from "./report";
  import NodeLayout from "./NodeLayout.svelte";
  import { AMINO_ACIDS } from "./types";
  import Node from "./Node.svelte";
  import App from "./App.svelte";
  export let params: { node_id: string };
  $: node_id = params.node_id;
  $: node = REPORT[node_id];
  console.log("FIRE", topologicalOrder());
</script>

<NodeLayout {node_id}>
  <h1>Sequences!</h1>
  <ul>
    {#each Object.entries(REPORT) as [other_node_id, other_node]}
      {#if other_node.kind == "leaf" && nodeDepends(other_node_id, node_id)}
        <li>
          <a href="#/nodes/{other_node_id}">
            Hello
            {#if ReportMetadata.sequences[other_node.sequence_id]}
              {ReportMetadata.sequences[other_node.sequence_id].label} ({ReportMetadata
                .sequences[other_node.sequence_id].sublabel})
            {:else}
              {other_node.sequence_id}
            {/if}
          </a>
        </li>
      {/if}
    {/each}
  </ul>
  <h1>Parents</h1>
  <ul>
    {#each node.parents as other_node_id}
      <li>
        <a href="#/nodes/{other_node_id}">{other_node_id}</a>
      </li>
    {/each}
  </ul>
  <h1>Children</h1>
  <ul>
    {#each Object.entries(REPORT) as [other_node_id, other_node]}
      {#if other_node.parents.includes(node_id)}
        <li>
          <a href="#/nodes/{other_node_id}">
            {#if other_node.kind == "leaf"}
              {other_node.sequence_id}
            {:else}
              {other_node_id}
            {/if}
          </a>
        </li>
      {/if}
    {/each}
  </ul>
</NodeLayout>

<style>
  .change-none {
    visibility: hidden;
  }
  .change-insert {
    background-color: aquamarine;
  }
  .change-delete {
    background-color: crimson;
  }
  .change-change {
    background-color: bisque;
  }
  .change {
    text-align: center;
    width: 3rem;
    color: black;
  }
</style>
