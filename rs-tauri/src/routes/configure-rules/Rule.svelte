<script lang="ts">
    import type { BlockDefinition, RuleDefinition } from "$lib/types";
    import Block from "./Block.svelte";
    import type { DragEventHandler } from "svelte/elements";
    import { deserializeBlockDefinition } from "$lib/serde";
    import { squeeze } from "$lib/transition";

    type Props = {
        rule: RuleDefinition;
        onRuleChanged: (rule: RuleDefinition) => void;
    };

    const { rule, onRuleChanged }: Props = $props();

    let curId = 0;
    const ids = new WeakMap();

    const blockId = (block: BlockDefinition) => {
        const id = ids.get(block);
        if (id) {
            return id;
        }
        curId += 1;
        const newId = curId;
        ids.set(block, newId);
        return newId;
    };

    let dropZone: HTMLDivElement;
    let dragInfo = $state<{
        rc: number;
        positions: number[];
        dropIndex: number;
    }>();
    let indexToRemove = $state<number>();

    const ondragenter: DragEventHandler<HTMLDivElement> = (event) => {
        if (indexToRemove !== undefined) {
            // Destroying event ondragstart would kill both the element and event.
            // But this always fires right after due to starting the drag on itself.
            onRuleChanged({
                ...rule,
                blocks: rule.blocks.toSpliced(indexToRemove, 1),
            });
            indexToRemove = undefined;
        }

        const dt = event.dataTransfer!;
        if (dt.types.includes("text/x-poehacks-block")) {
            event.preventDefault();
            if (dragInfo) {
                dragInfo.rc += 1;
            } else {
                dragInfo = {
                    rc: 1,
                    positions: [...dropZone.children].map(
                        (child) => child.getBoundingClientRect().y,
                    ),
                    dropIndex: rule.blocks.length,
                };
            }
        }
    };

    const ondragover: DragEventHandler<HTMLDivElement> = (event) => {
        const dt = event.dataTransfer!;
        if (dt.types.includes("text/x-poehacks-block") && dragInfo) {
            event.preventDefault();
            dragInfo.dropIndex =
                dragInfo.positions.findLastIndex(
                    (pos) => pos < event.clientY,
                ) ?? rule.blocks.length;
        }
    };

    const ondrop: DragEventHandler<HTMLDivElement> = (event) => {
        const dt = event.dataTransfer!;
        const data = dt.getData("text/x-poehacks-block");
        if (data && dragInfo) {
            event.preventDefault();
            const block = deserializeBlockDefinition(data);
            onRuleChanged({
                ...rule,
                blocks: rule.blocks.toSpliced(dragInfo.dropIndex, 0, block),
            });
            dragInfo = undefined;
        }
    };

    const ondragleave: DragEventHandler<HTMLDivElement> = (event) => {
        const dt = event.dataTransfer!;
        if (dt.types.includes("text/x-poehacks-block")) {
            event.preventDefault();
            if (dragInfo) {
                dragInfo.rc -= 1;
                if (!dragInfo.rc) {
                    dragInfo = undefined;
                }
            }
        }
    };
</script>

<div class="rule">
    <header>
        Rule&nbsp;
        <input
            type="text"
            value={rule.name}
            onkeyup={(e) =>
                onRuleChanged({ ...rule, name: e.currentTarget.value })}
            disabled={!rule.id}
        /><span>#{rule.id}</span>
    </header>
    <div
        bind:this={dropZone}
        {ondragenter}
        {ondragover}
        {ondrop}
        {ondragleave}
        role="rowgroup"
        tabindex="-1"
    >
        {#each rule.blocks as block, i (blockId(block))}
            <div
                class:gapTop={i === dragInfo?.dropIndex}
                out:squeeze={{ duration: 200 }}
            >
                <Block
                    {block}
                    onBlockMoved={() => {
                        indexToRemove = rule.blocks.indexOf(block);
                    }}
                    onBlockChanged={(newBlock) => {
                        ids.set(newBlock, blockId(block));
                        onRuleChanged({
                            ...rule,
                            blocks: rule.blocks.toSpliced(i, 1, newBlock),
                        });
                    }}
                />
            </div>
        {/each}
        <div class="dropNew" class:gapTop={!!dragInfo}></div>
    </div>
</div>

<style>
    .rule {
        padding: 0.5em;
        box-shadow: 4px 4px 8px color-mix(in hsl, var(--accent-rule), #000 50%);
    }

    .rule,
    input {
        background-color: var(--accent-rule);
        color: var(--accent-rule-text);
        font-size: large;
    }

    div[role="rowgroup"] {
        margin: 0.5em 0.5em;
        border: 0.1em dashed
            color-mix(in hsl, var(--accent-rule), var(--accent-rule-text) 50%);
        min-height: 4em;
        padding: 0.25em;
    }

    header {
        display: flex;
        align-items: center;
    }

    input,
    span {
        font-family: "Courier New", Courier, monospace;
    }

    span {
        border-width: 2px 2px 2px 0;
        border-color: #000;
        border-style: ridge;
        background-color: color-mix(
            in hsl,
            var(--accent-rule),
            var(--accent-rule-text) 25%
        );
    }

    div[role="rowgroup"] > div {
        transition:
            margin-top 100ms,
            margin-bottom 100ms;
    }
    .gapTop {
        margin-top: 2.5em;
    }
</style>
