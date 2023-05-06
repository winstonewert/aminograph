<script lang="ts">
	export let title: string
</script>
<style>
  .card {
    margin: 1rem;
  }
  .card-header {
	  border-bottom: solid 1px white;
	  background-color: white;
  }
  .card-header-title {
	  color: black;
  }
 
</style>
<div class="card">
	<div class="card-header">
		<div class="card-header-title">
			{title}
		</div>
	</div>
	<div class="card-content">
		<div class="content">
			<slot/>
		</div>
	</div>
</div>
