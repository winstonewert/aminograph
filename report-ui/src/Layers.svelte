<script lang="ts">
  import type {Report} from "./types"
  import Tree from "./Tree.svelte"
  import {serializePlan} from "./serialize-plan"
  export let report: Report
  export let path: string[]

  $: trace_index = parseInt(path[0], 10)
</script>

<style>
  .layers {
    display: flex;
    overflow: auto;
    flex-direction: column
  }
  .tree {
    overflow: auto;
    flex-grow: 1;
  }
  .button.active {
    border: solid 2px white;
  }
</style>

<div class="layers">
  <div class="tree">
    <Tree report={report} entries={serializePlan(report, report.plan.target, trace_index, null)} let:entry={entry}>
      {#if entry.parent_index !== undefined}
      {#each report.positions.map((position, index) => ({...position, index})).filter((position) => 
      position.amino_acids[entry.index] != position.amino_acids[entry.parent_index]
      && position.primary[entry.index] == '1'
      && position.primary[entry.parent_index] == '1'
      && position.amino_acids[entry.index] != '-'
      ) as position}
      <span class="tag">
        {position.amino_acids[entry.parent_index]}{position.index+1}{position.amino_acids[entry.index]}
      </span>
      {/each}
      {/if}
    </Tree>
  </div>
</div>