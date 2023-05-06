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
  <div class="contain">
    <h1>Sequences</h1>
    <ul>
      {#each Object.entries(REPORT) as [other_node_id, other_node]}
        {#if other_node.kind == "leaf" && nodeDepends(other_node_id, node_id)}
          <li>
            <a href="#/nodes/{other_node_id}">
              {#if ReportMetadata.sequences[other_node.sequence_id]}
                {#if ReportMetadata.sequences[other_node.sequence_id].image}
                  <img
                    class="sequence-image"
                    src={ReportMetadata.sequences[other_node.sequence_id].image}
                    alt=""
                  />
                {/if}
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
    <h1>Changes</h1>
    <ul>
      {#each node.amino_acids as amino_acid, index}
        {#if amino_acid != node_position_inherited(node_id, index).amino_acid}
          <li>
            <a class="change tag" href="#/sequence/{index}"
              >{node_position_inherited(node_id, index).amino_acid}{index +
                1}{amino_acid}

              {Object.keys(REPORT).filter(
                (other_id) =>
                  nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf" &&
                  REPORT[other_id].amino_acids[index] === amino_acid
              ).length}/{Object.keys(REPORT).filter(
                (other_id) =>
                  nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf" &&
                  (REPORT[other_id].amino_acids[index] === "X" ||
                    REPORT[other_id].amino_acids[index] == "-")
              ).length}/{Object.keys(REPORT).filter(
                (other_id) =>
                  nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf"
              ).length}
              {Object.keys(REPORT).filter(
                (other_id) =>
                  !nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf" &&
                  REPORT[other_id].amino_acids[index] === amino_acid
              ).length}/{Object.keys(REPORT).filter(
                (other_id) =>
                  !nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf" &&
                  (REPORT[other_id].amino_acids[index] === "X" ||
                    REPORT[other_id].amino_acids[index] == "-")
              ).length}/{Object.keys(REPORT).filter(
                (other_id) =>
                  !nodeDepends(other_id, node_id) &&
                  REPORT[other_id].kind === "leaf"
              ).length}
            </a>
          </li>
        {/if}
      {/each}
    </ul>
  </div>
</NodeLayout>

<style>
  .contain {
    overflow-y: auto;
    width: 100%;
    flex-grow: 1;
  }
  .sequence-image {
    width: 16px;
    height: 16px;
  }
</style>
