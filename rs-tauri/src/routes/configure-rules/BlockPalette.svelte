<script lang="ts">
    import type { BlockDefinition } from "$lib/types";
    import {
        BLOCK_CLICK_VARIABLES,
        BLOCK_SCROLL_VARIABLES,
        BLOCK_WHEN_CONDITIONS,
        BLOCK_WHEN_VARIABLES,
    } from "$lib/constants";

    import Block from "./Block.svelte";

    const BLOCKS_CONDITION = {
        when: {
            kind: "when",
            variable: BLOCK_WHEN_VARIABLES[0],
            condition: BLOCK_WHEN_CONDITIONS[0],
            value: "50%",
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
        click: {
            kind: "click",
            variable: BLOCK_CLICK_VARIABLES[0],
            value: "1",
        },
        scroll: {
            kind: "scroll",
            variable: BLOCK_SCROLL_VARIABLES[0],
            value: "1",
        },
    } satisfies Partial<Record<BlockDefinition["kind"], BlockDefinition>>;

    const BLOCKS_TIMING = {
        cooldown: {
            kind: "cooldown",
            value: "1s",
        },
    } satisfies Partial<Record<BlockDefinition["kind"], BlockDefinition>>;

    const assertAllBlocks: Record<BlockDefinition["kind"], BlockDefinition> = {
        ...BLOCKS_CONDITION,
        ...BLOCKS_ACTION,
        ...BLOCKS_TIMING,
    };
    assertAllBlocks;
</script>

<div class="rtl">
    <div class="ltr">
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
    </div>
</div>

<style>
    h1 {
        font-size: 1em;
        text-align: left;
        margin: 0;
    }
    .rtl {
        padding: 0.5em;
        overflow-y: scroll;
        box-shadow: inset 0px 0px 8px #777;
        direction: rtl;
        width: fit-content;
    }
    .ltr {
        direction: ltr;
        display: flex;
        flex-direction: column;
        gap: 0.5em;
    }
</style>
