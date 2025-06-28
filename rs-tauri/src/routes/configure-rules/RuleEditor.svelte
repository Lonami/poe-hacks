<script lang="ts">
    import Rule from "./Rule.svelte";
    import BlockPalette from "./BlockPalette.svelte";
    import type { RuleDefinition } from "$lib/types";
    import { mkGetObjectKey } from "$lib/keys";

    let ruleDefinitions = $state<RuleDefinition[]>([]);

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
        ></BlockPalette>
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
        flex-shrink: 0;
        display: flex;
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
