<script lang="ts">
    import type { BlockDefinition } from "$lib/types";
    import {
        BLOCK_CLICK_VARIABLES,
        BLOCK_SCROLL_VARIABLES,
        BLOCK_STAT_CONDITIONS,
        BLOCK_STAT_VARIABLES,
    } from "$lib/constants";

    import Block from "./Block.svelte";

    type Props = {
        onAddNewRule: () => void;
        onResetAllRules: () => void;
    };

    const { onAddNewRule, onResetAllRules }: Props = $props();

    const BLOCKS_CONDITION = {
        stat: {
            kind: "stat",
            variable: BLOCK_STAT_VARIABLES[0],
            condition: BLOCK_STAT_CONDITIONS[0],
            value: "50%",
        },
        key: {
            kind: "key",
            value: "Spc",
        },
        mouse: {
            kind: "mouse",
            variable: BLOCK_CLICK_VARIABLES[0],
        },
    } satisfies Partial<Record<BlockDefinition["kind"], BlockDefinition>>;

    const BLOCKS_ACTION = {
        press: {
            kind: "press",
            value: "1",
        },
        type: {
            kind: "type",
            value: "/hideout",
        },
        disconnect: {
            kind: "disconnect",
        },
        click: {
            kind: "click",
            variable: BLOCK_CLICK_VARIABLES[0],
        },
        scroll: {
            kind: "scroll",
            variable: BLOCK_SCROLL_VARIABLES[0],
        },
    } satisfies Partial<Record<BlockDefinition["kind"], BlockDefinition>>;

    const BLOCKS_TIMING = {
        cooldown: {
            kind: "cooldown",
            value: "1s",
        },
        delay: {
            kind: "delay",
            value: "250ms",
        },
    } satisfies Partial<Record<BlockDefinition["kind"], BlockDefinition>>;

    const assertAllBlocks: Record<BlockDefinition["kind"], BlockDefinition> = {
        ...BLOCKS_CONDITION,
        ...BLOCKS_ACTION,
        ...BLOCKS_TIMING,
    };
    assertAllBlocks;
</script>

<h1>Rules</h1>
<div>
    <button onclick={onAddNewRule}>Add new rule</button>
    <button
        class="danger"
        onclick={() => {
            if (confirm("Reset all rules? This cannot be undone.")) {
                onResetAllRules();
            }
        }}>Reset</button
    >
</div>
<h1>Events</h1>
{#each Object.entries(BLOCKS_CONDITION) as [kind, block] (kind)}
    <Block {block} isReadonly />
{/each}
<h1>Actions</h1>
{#each Object.entries(BLOCKS_ACTION) as [kind, block] (kind)}
    <Block {block} isReadonly />
{/each}
<h1>Timing</h1>
{#each Object.entries(BLOCKS_TIMING) as [kind, block] (kind)}
    <Block {block} isReadonly />
{/each}

<style>
    h1 {
        font-size: 1em;
        text-align: left;
        margin: 0;
    }
</style>
