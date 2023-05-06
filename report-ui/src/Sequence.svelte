<script lang="ts">
  import REPORT, {sequencePositions} from "./report"
  console.log(sequencePositions())
</script>


<div class="node-list container">
  <ul>
    {#each sequencePositions() as position, index}
      <li>
        <a class="button" href="#/sequence/{index}">
          <span class="label">
            {index+1}
			<div class="logo">
				{#each position.logo as item}
					<div class="tag logo-item amino-acid-{item.amino_acid}">
						{item.amino_acid} ({item.count})
					</div>
				{/each}
			</div>
          </span>
        </a>
      </li>
    {/each}
  </ul>
</div>

<style>
.logo {
	display: inline-block;
}
	.logo-item {
		display: inline-block;
		margin-left: 1em;
	}
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
  	.change-table {

	}
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
  .card {
    margin: 1rem;
  }
  .amino-acid {
						border: solid 1px black;
						border-radius: 9999px;
						padding-left: .5rem;
						padding-right: .5rem;
  }

.amino-acid-A {
						background-color: #5B0099;
						color: white;
					}

					.amino-acid-R {
						background-color: #66e600;
					}

					.amino-acid-N {
						background-color: #0000ff;
						color: white;
						;
					}

					.amino-acid-D {
						background-color: #0082ff;
						;
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
						background-color: #77019A;
						color: white;
					}

					.amino-acid-H {
						background-color: #B39500;
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
						background-color: #02009B;
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

          .amino-acid-change {

						border-width: 2px;
						border-color: red;
          }
</style>
