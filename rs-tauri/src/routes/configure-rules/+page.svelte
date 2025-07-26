<script lang="ts">
  import Rule from "./Rule.svelte";
  import BlockPalette from "./BlockPalette.svelte";
  import type { RuleDefinition } from "$lib/types";
  import { mkGetObjectKey } from "$lib/keys";
  import { G } from "$lib/globals.svelte";

  const idx = $derived(G.profiles.findIndex((p) => p.active));
  const ruleDefinitions = $derived(G.profiles[idx].rules);

  const ruleId = mkGetObjectKey<RuleDefinition>();
</script>

<div class="editor">
  <div class="palette">
    <BlockPalette
      onAddNewRule={() => {
        ruleDefinitions.push({ name: "", blocks: [] });
      }}
      onResetAllRules={() => {
        ruleDefinitions.length = 0;
      }}
    />
  </div>

  <div role="grid">
    {#each ruleDefinitions as rule, i (ruleId(rule))}
      <Rule
        bind:rule={ruleDefinitions[i]}
        onDeleteRule={() => {
          ruleDefinitions.splice(i, 1);
        }}
      />
    {/each}
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-grow: 1;
    overflow: hidden;
  }

  .palette {
    box-shadow: inset 0px 0px 8px #777;
    padding: 0.5em;
    min-width: fit-content;
    display: flex;
    flex-direction: column;
    gap: 0.25em;
    overflow-y: auto;
  }

  div[role="grid"] {
    padding: 2em;
    display: flex;
    gap: 1em;
    overflow-x: auto;
    flex-grow: 1;
    align-items: start;
  }
</style>
