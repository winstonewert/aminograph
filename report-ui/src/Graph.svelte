<script lang="ts">
  import Nodes from "./Nodes.svelte";
  import REPORT, { nodeDepends, reportLayers, ReportMetadata } from "./report";
  import { serializeSequenceDependencies } from "./serialize-dependencies";
  export let focus: string | undefined;
  $: graph = reportLayers(focus);

  function positionSegment(start, end) {
    const length = Math.sqrt(
      (start.x - end.x) * (start.x - end.x) +
        (start.y - end.y) * (start.y - end.y)
    );
    const center_x = (start.x + end.x - length) / 2;
    const center_y = (start.y + end.y - 1) / 2;

    const angle = Math.atan2(start.y - end.y, start.x - end.x);
    return `width: ${length}px; left: ${center_x}px; top: ${center_y}px;transform: rotate(${angle}rad)`;
  }
</script>

<div class="graph">
  {#each graph.nodes() as node_id}
    <a
      href="#/nodes/{node_id}/graph"
      class="node"
      class:leaf={REPORT[node_id].kind == "leaf"}
      style="left: {graph.node(node_id).x}px; top: {graph.node(node_id).y}px"
    >
      {#if REPORT[node_id].kind === "leaf"}
        {#if ReportMetadata.sequences[REPORT[node_id].sequence_id]}
          <div class="sequence-label">
            {ReportMetadata.sequences[REPORT[node_id].sequence_id].label}
          </div>
          <div class="sequence-sublabel">
            {ReportMetadata.sequences[REPORT[node_id].sequence_id].sublabel}
          </div>
        {:else}
          {REPORT[node_id].sequence_id}
        {/if}
      {:else}
        {node_id}
      {/if}
    </a>
  {/each}
  {#each graph.edges() as edge}
    {#each graph.edge(edge).points.slice(1) as end, index}
      <div
        class="segment"
        style={positionSegment(graph.edge(edge).points[index], end)}
      />
    {/each}
  {/each}
</div>

<style>
  .graph {
    position: relative;
    overflow: auto;
    height: 100%;
  }
  .node {
    position: absolute;
    border: solid 1px black;
    width: 100px;
    height: 100px;
    margin-left: -25px;
    margin-top: -25px;
    text-align: center;
    line-height: 50px;
    border-radius: 9999px;
    width: 50px;
    height: 50px;
    z-index: 1;
    background-color: #40e0d0;
    color: black;
  }
  .node.leaf {
    border-radius: inherit;
    width: 200px;
    height: 100px;
    margin-left: -100px;
    margin-top: -25px;
  }
  .sequence-sublabel {
    font-size: x-small;
  }
  .segment {
    position: absolute;
    height: 2px;
    background-color: white;
  }
</style>
