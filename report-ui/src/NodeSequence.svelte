<script lang="ts">
  import REPORT, {
    allDependencies,
    changeMode,
    interestingPositions,
    node_position_inherited,
    topologicalOrder,
  } from "./report";
  import NodeLayout from "./NodeLayout.svelte";
  import { AMINO_ACIDS } from "./types";
  import Node from "./Node.svelte";
  export let params: { node_id: string };
  $: node_id = params.node_id;
  $: node = REPORT[node_id];
</script>

<NodeLayout {node_id}>
  <table>
    <thead>
      <tr>
        <th />
        <th />
        {#each allDependencies(node_id) as dependency_id}
          <th><a href="#/nodes/{dependency_id}">{dependency_id}</a></th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each interestingPositions(node_id) as index}
        <tr>
          <th>{index + 1}</th>
          <td>{node.amino_acids[index]}</td>
          {#each allDependencies(node_id) as dependency_id}
            <td class="change change-{changeMode(dependency_id, index)}"
              >{REPORT[dependency_id].amino_acids[index]}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
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
  th {
    position: sticky;
    background-color: #2b3e50;
  }
  thead th {
    top: 0;
  }
</style>
