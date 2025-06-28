<script lang="ts">
    import Rule from "./Rule.svelte";
    import BlockPalette from "./BlockPalette.svelte";
    import type { RuleDefinition } from "$lib/types";

    let ruleDefinitions = $state<RuleDefinition[]>([
        {
            id: 0,
            name: "Create new rule…",
            blocks: [],
        },
    ]);
</script>

<div class="editor">
    <div class="palette">
        <BlockPalette></BlockPalette>
    </div>

    <div role="grid">
        {#each ruleDefinitions as rule, i (rule.id)}
            <Rule
                {rule}
                onRuleChanged={(rule) => {
                    if (rule.id) {
                        ruleDefinitions[i] = rule;
                    } else {
                        ruleDefinitions.splice(ruleDefinitions.length - 1, 0, {
                            id:
                                Math.max(...ruleDefinitions.map((r) => r.id)) +
                                1,
                            name: "",
                            blocks: rule.blocks,
                        });
                    }
                    const emptyIndex = ruleDefinitions.findIndex(
                        (r, j) => j !== i && r.id && !r.blocks.length,
                    );
                    if (emptyIndex !== -1) {
                        ruleDefinitions.splice(emptyIndex, 1);
                    }
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
        overflow-x: visible;
        flex-grow: 1;
    }
</style>
