<script lang="ts">
  import type {Report, TreeEntry} from "./types"
  import Tree from "./Tree.svelte"
  import {serializePlan} from "./serialize-plan"
  import {serializeSequenceDependencies} from "./serialize-dependencies"
  import Card from "./Card.svelte"
  import {findChanges} from "./find-changes"
  export let report: Report
  export let path: string[]

  $: sequence_index = parseInt(path[0],10)
</script>

<style>
  .layers {
    display: flex;
    overflow: auto;
    flex-direction: column
  }
  .buttons {
    overflow: auto;
    flex-wrap: nowrap;
    flex-shrink: 0;
  }
  .trees {
    overflow: auto;
    flex-grow: 1;
    display: flex;
    flex-wrap: wrap;
  }
  .button.active {
    border: solid 2px white;
  }
</style>

<div class="layers">
  <div>
    Sequence:
  <div class="select">
    <select value={sequence_index} on:change={(event) => {
      window.location.href = "#/sequences/" + event.target.value
    }}>
    {#each report.sequences as sequence, index}
    <option value={index}>{sequence.id}</option>
    {/each}
    </select>
  </div>
</div>
  <div class="trees">
	<Card title="Dependencies">
		<Tree report={report} entries={serializeSequenceDependencies(report, sequence_index)} let:entry={entry}>
			{#each findChanges(report, entry) as position}
				<span class="tag">
					{position.parent_amino_acid}{position.index+1}{position.amino_acid}
				</span>
			{/each}
		</Tree>
	</Card>
 </div>
</div>