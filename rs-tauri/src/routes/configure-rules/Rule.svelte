<script lang="ts">
    import type { BlockDefinition, RuleDefinition } from "$lib/types";
    import Block from "./Block.svelte";
    import type { DragEventHandler } from "svelte/elements";
    import { mkGetObjectKey } from "$lib/keys";
    import { slide } from "svelte/transition";

    type Props = {
        rule: RuleDefinition;
        onDeleteRule: () => void;
    };

    const { rule = $bindable(), onDeleteRule }: Props = $props();

    const blockId = mkGetObjectKey<BlockDefinition>();

    let dropZone: HTMLDivElement;
    let dragInfo = $state<{
        rc: number;
        positions: number[];
        dropIndex: number;
    }>();
    let indexToRemove = $state<number>();

    const ondragenter: DragEventHandler<HTMLDivElement> = (event) => {
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
            if (dt.effectAllowed === "copyMove") {
                dt.dropEffect = event.ctrlKey ? "copy" : "move";
            }
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
            const block = JSON.parse(data);

            rule.blocks.splice(dragInfo.dropIndex, 0, block);

            if (!event.ctrlKey && indexToRemove !== undefined) {
                rule.blocks.splice(
                    indexToRemove >= dragInfo.dropIndex
                        ? indexToRemove + 1
                        : indexToRemove,
                    1,
                );
                indexToRemove = undefined;
            }

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

<div
    class="rule"
    in:slide={{ duration: 200, axis: "x" }}
    out:slide={{ duration: 200, axis: "x" }}
>
    <header>
        Rule
        <input
            type="text"
            value={rule.name}
            onkeyup={(e) => {
                rule.name = e.currentTarget.value;
            }}
        />
        <button aria-label="Delete rule" onclick={onDeleteRule}>✕</button>
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
                in:slide={{ duration: 200 }}
                out:slide={{ duration: 200 }}
            >
                <Block
                    bind:block={rule.blocks[i]}
                    onBlockMoved={() => {
                        indexToRemove = i;
                    }}
                    onBlockDeleted={() => {
                        rule.blocks.splice(i, 1);
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
        transition: width 200ms;
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
        min-height: 2.5em;
        padding: 0.25em;
    }

    header {
        display: flex;
        gap: 0.5em;
        align-items: center;
    }

    input {
        font-family: "Courier New", Courier, monospace;
        flex-grow: 1;
        width: 12ch;
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
