<script lang="ts">
  import url from "./url";
  import type { Report } from "./types";
  import Nodes from "./Nodes.svelte";
  import Grid from "./Grid.svelte";
  import Node from "./Node.svelte";
  import Router, { location } from "svelte-spa-router";
  import Home from "./Home.svelte";
  import NavTab from "./NavTab.svelte";
  import NodeSequence from "./NodeSequence.svelte";
  import Graph from "./GraphPage.svelte";
  import NodeGraph from "./NodeGraph.svelte";
  import Sequence from "./Sequence.svelte";
  import SequenceItem from "./SequenceItem.svelte";
    import SequenceItemGraph from "./SequenceItemGraph.svelte";


  $: pathSegments = $url.substring(2).split("/");

  const routes = {
    "/": Home,
    "/grid/": Grid,
    "/graph/": Graph,
    "/nodes": Nodes,
    "/nodes/:node_id": Node,
    "/nodes/:node_id/sequence": NodeSequence,
    "/nodes/:node_id/graph": NodeGraph,
    "/sequence": Sequence,
    "/sequence/:index": SequenceItem,
    "/sequence/:index/graph": SequenceItemGraph,

  };
</script>

<div class="app">
  <div class="tabs">
    <ul>
      <NavTab path="grid" name="Grid" />
      <NavTab path="nodes" name="Nodes" />
      <NavTab path="graph" name="Graph" />
      <NavTab path="sequence" name="Sequence" />
    </ul>
  </div>
  <Router {routes} />
</div>

<style>
  .app {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }
  .tabs {
    flex-shrink: 0;
  }
</style>
