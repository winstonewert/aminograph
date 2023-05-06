<script lang="ts">
  import REPORT, { nodeDepends } from "./report";
  import Fa from "svelte-fa";
  import {
    faDna,
    faCodeBranch,
    faStar,
  } from "@fortawesome/free-solid-svg-icons";
  import Node from "./Node.svelte";
</script>

<div class="node-list container">
  <ul>
    {#each Object.entries(REPORT) as [node_id, node]}
      <li>
        <a class="button" href="#/nodes/{node_id}">
          {#if node.kind == "leaf"}
            <Fa icon={faDna} />
          {:else if node.kind == "other"}
            <Fa icon={faCodeBranch} />
          {:else}
            <Fa icon={faStar} />
          {/if}
          <span class="label">
            {node.sequence_id || node_id}
          </span>
        </a>
      </li>
    {/each}
  </ul>
</div>

<style>
  .node-list {
    flex-grow: 1;
    overflow: auto;
  }
  .node-list ul {
    columns: 3;
  }

  .button {
    margin: 0.25rem;
    width: 100%;
    text-align: left;
    justify-content: left;
  }
  .label {
    margin-left: 0.25rem;
  }
</style>
