<script lang="ts">
  import REPORT, {
    sequencePositions,
    reportLayers,
    changeMode,
    node_position_inherited,
  } from "./report";
  import SequenceItemLayout from "./SequenceItemLayout.svelte";

  export let params: { index: string };
  $: index = parseInt(params.index, 10);
  const graph = reportLayers(undefined);

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

<SequenceItemLayout {index}>
  <div class="graph">
    <h2>Sequences</h2>
    <div class="rows">
      {#each Object.values(REPORT) as node}
        {#if node.kind == "leaf"}
          <div
            class="amino-acid amino-acid-{node.amino_acids[index]}"
            title={node.sequence_id}
          >
            {node.amino_acids[index]}
          </div>
        {/if}
      {/each}
    </div>
    <h2>Changes</h2>
    <div class="rows">
      {#each Object.entries(REPORT) as [node_id, node]}
        {#if changeMode(node_id, index) != "none"}
          <div class="badge amino-acid-{node.amino_acids[index]}">
            {node_position_inherited(node_id, index).amino_acid} -> {node
              .amino_acids[index]} @
            <a href="#/nodes/{node_id}">
              {#if node.kind == "leaf"}
                {node.sequence_id}
              {:else}
                {node_id}
              {/if}
            </a>
          </div>
        {/if}
      {/each}
    </div>
  </div>
</SequenceItemLayout>

<style>
  .graph {
    position: relative;
    overflow: auto;
    height: 100%;
  }
  .node {
    position: absolute;
    border: solid 1px black;
    margin-left: -25px;
    margin-top: -25px;
    text-align: center;
    line-height: 10px;
    padding-top: 20px;
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
    height: 50px;
    margin-left: -100px;
    margin-top: -25px;
  }
  .segment {
    position: absolute;
    height: 2px;
    background-color: white;
  }
  .amino-acid {
    display: inline-block;
    margin-top: 5px;
    width: 1rem;
    text-align: center;
  }
  .amino-acid-A {
    background-color: #5b0099;
    color: white;
  }

  .amino-acid-R {
    background-color: #66e600;
  }

  .amino-acid-N {
    background-color: #0000ff;
    color: white;
  }

  .amino-acid-D {
    background-color: #0082ff;
  }

  .amino-acid-C {
    background-color: #dc0097;
  }

  .amino-acid-Q {
    background-color: #018100;
  }

  .amino-acid-E {
    background-color: #00c801;
  }

  .amino-acid-G {
    background-color: #77019a;
    color: white;
  }

  .amino-acid-H {
    background-color: #b39500;
  }

  .amino-acid-I {
    background-color: #e63101;
  }

  .amino-acid-L {
    background-color: #e66500;
  }

  .amino-acid-K {
    background-color: #8b5a00;
  }

  .amino-acid-M {
    background-color: #c80000;
  }

  .amino-acid-F {
    color: black;
    background-color: #ffff00;
  }

  .amino-acid-P {
    background-color: #02009b;
    color: white;
  }

  .amino-acid-S {
    color: black;
    background-color: #01ffff;
  }

  .amino-acid-T {
    background-color: #8c8c8c;
  }

  .amino-acid-W {
    background-color: #633c02;
    color: white;
  }

  .amino-acid-Y {
    background-color: #5c2400;
    color: white;
  }

  .amino-acid-V {
    background-color: #b4009a;
    color: white;
  }
</style>
