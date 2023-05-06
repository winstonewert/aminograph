<script lang="ts">
  import REPORT, { nodeDepends, ReportMetadata } from "./report";
</script>

<div class="table-container table">
  <table>
    <thead>
      <tr>
        <th class="corner" />
        {#each Object.entries(REPORT).filter((node) => node[1].kind != "leaf") as [node_id, node]}
          <th><a href="#/nodes/{node_id}">{node_id}</a></th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each Object.entries(REPORT).filter((node) => node[1].kind == "leaf") as [sequence_id, sequence]}
        <tr>
          <th>
            {#if ReportMetadata.sequences[sequence.sequence_id]?.image}
              <img
                class="sequence-image"
                src={ReportMetadata.sequences[sequence.sequence_id].image}
                alt=""
              />
            {/if}
            <div class="sequence-label">
              <a href="#/nodes/{sequence_id}"
                >{ReportMetadata.sequences[sequence.sequence_id]?.label ||
                  sequence.sequence_id}</a
              >
              <div class="sequence-sublabel">
                {ReportMetadata.sequences[sequence.sequence_id]?.sublabel || ""}
              </div>
            </div>
          </th>
          {#each Object.entries(REPORT).filter((node) => node[1].kind != "leaf") as [node_id, node]}
            <td class:present={nodeDepends(sequence_id, node_id)} />
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  td {
    background-color: crimson;
  }
  td.present {
    background-color: darkseagreen;
  }

  .table-container {
    overflow: scroll;
  }

  th {
    position: sticky;
    background-color: #2b3e50;
  }

  thead th {
    top: 0;
    writing-mode: vertical-lr;
  }

  tbody th {
    left: 0;
    white-space: nowrap;
  }

  .corner {
    left: 0;
    z-index: 1;
  }

  td {
    border: solid 1px black;
  }

  .sequence-image {
    display: inline-block;
    width: 32px;
    height: 32px;
  }
  .sequence-label {
    display: inline-block;
  }
</style>
