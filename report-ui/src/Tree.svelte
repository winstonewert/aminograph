<script lang="ts">
  import type {TreeEntry, Report} from "./types"

  type Entry = $$Generic<TreeEntry>;

  interface $$Slots {
	  default: {entry: Entry}
  }

  export let report: Report
  export let entries: Entry[]
</script>

<style>
	.tree {
		margin-left: 1rem;
		margin-right: 1rem;
	}
	.entry {
		height: 1.2rem;
		white-space: nowrap;
	}

	.entry .entry-label {
		border: solid 1px black;
		padding-left: .25rem;
		padding-right: .25rem;
		border-radius: .375rem;
		position: relative;
		left: -.5rem;
		z-index: 10;
	}

	.alternate-label {
		border: solid 1px black;
		padding-left: .25rem;
		padding-right: .25rem;
		border-radius: .375rem;
		position: relative;
		left: -.5rem;
		z-index: 10;	
		background-color: tomato;
	}

	.alternate-label.active {
		font-weight: bold;
		color: white;
	}

	.entry-Node .entry-label {
		color: black;
		background-color: aquamarine;
	}
	.entry-Module .entry-label {
		color: black;
		background-color: dodgerblue;
	}
	.entry-Sequence .entry-label {
		color: black;
		background-color: bisque;
	}
	.indent {
		width: 1rem;
		height: 1.2rem;
		display: inline-block;
						position: relative;
						top: -.3rem;
	
	}
	.indent-join {
		border-bottom: solid 1px white;
	}
	.indent-join, .indent-line {
		border-left: solid 1px white;
	}
</style>

<div class="tree">
{#each entries as entry}
<div class="entry entry-{entry.action.type}">
	{#each entry.indent as indent}
		<span class="indent indent-{indent}"></span>
	{/each}
	{#if entry.action.type == "Node" || entry.action.type == "Module" || entry.action.type == "Combine"}
	<a class="entry-label" href={"#/nodes/" + entry.action.value} >
		{entry.action.value}
	</a>
	{#each entry.alternates as alternate, alternate_index}
	<a class="alternate-label" class:active={alternate == entry.index} href={"#/trace/" + alternate} >
		A{alternate_index}
	</a>
	{/each}
	{/if}
	{#if entry.action.type == "Sequence"}
	<span class="entry-label">
		{report.sequences[entry.action.value].id}
	</span>
	{/if}
	<slot entry={entry}></slot>
</div>
{/each}
</div>