<script lang="ts">
  import NodeLayout from "./NodeLayout.svelte";

  import Graph from "./Graph.svelte"
  export let params: { node_id: string };
  $: node_id = params.node_id;
</script>


<NodeLayout {node_id}>
  <Graph focus={node_id}/>
</NodeLayout>
