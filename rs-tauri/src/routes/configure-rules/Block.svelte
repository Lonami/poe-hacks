<script lang="ts">
    import type { BlockDefinition } from "$lib/types";
    import {
        BLOCK_CLICK_VARIABLES,
        BLOCK_SCROLL_VARIABLES,
        BLOCK_WHEN_CONDITIONS,
        BLOCK_WHEN_VARIABLES,
    } from "$lib/constants";
    import { serializeBlockDefinition } from "$lib/serde";

    import type {
        DragEventHandler,
        KeyboardEventHandler,
    } from "svelte/elements";
    import GripAreaDisplay from "./GripAreaDisplay.svelte";

    type Props = {
        block: BlockDefinition;
    } & (
        | {
              onBlockMoved: () => void;
              onBlockChanged: (block: BlockDefinition) => void;
              isReadonly?: undefined;
          }
        | {
              onBlockMoved?: undefined;
              onBlockChanged?: undefined;
              isReadonly: true;
          }
    );

    const {
        block,
        onBlockMoved,
        onBlockChanged,
        isReadonly: disabled,
    }: Props = $props();

    const ondragstart: DragEventHandler<HTMLDivElement> = (event) => {
        const dt = event.dataTransfer!;
        dt.setData("text/x-poehacks-block", serializeBlockDefinition(block));
        dt.effectAllowed = disabled ? "copy" : "copyMove";
        onBlockMoved?.();
    };

    let accent = $derived.by(() => {
        switch (block.kind) {
            case "when":
                return "condition";
            case "press":
            case "type":
            case "click":
            case "scroll":
                return "action";
            case "cooldown":
                return "timing";
            default:
                const assertNever: never = block;
                throw assertNever;
        }
    });

    const onkeyup: KeyboardEventHandler<HTMLInputElement> = (event) => {
        onBlockChanged?.({ ...block, value: event.currentTarget.value });
    };
</script>

<div
    draggable="true"
    {ondragstart}
    role="row"
    tabindex="-1"
    style="--accent: var(--accent-{accent}); --text: var(--accent-{accent}-text); --length: {block
        .value.length + 2}ch;"
>
    <GripAreaDisplay height={"24"} />
    {#if block.kind === "when"}
        <strong>when</strong>
        <select value={block.variable} {disabled}>
            {#each BLOCK_WHEN_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <select value={block.condition} {disabled}>
            {#each BLOCK_WHEN_CONDITIONS as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else if block.kind === "press"}
        <strong>press</strong>
        <input
            type="text"
            value={block.value}
            maxlength="1"
            onkeydown={(e) => {
                e.preventDefault();
                let value = "";
                if (e.ctrlKey) {
                    value += "Ctrl+";
                }
                if (e.shiftKey) {
                    value += "Shift+";
                }
                if (e.altKey) {
                    value += "Alt+";
                }
                if (e.key.length === 1) {
                    value += e.key === " " ? "Spc" : e.key.toUpperCase();
                }
                onBlockChanged?.({ ...block, value });
            }}
            {disabled}
        />
    {:else if block.kind === "type"}
        <strong>type</strong>
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else if block.kind === "click"}
        <strong>click</strong>
        <select value={block.variable}>
            {#each BLOCK_CLICK_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <input
            type="number"
            value={block.value}
            min="1"
            max="99"
            {onkeyup}
            {disabled}
        />
        times
    {:else if block.kind === "scroll"}
        <strong>scroll</strong>
        <select value={block.variable}>
            {#each BLOCK_SCROLL_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <input
            type="number"
            value={block.value}
            min="1"
            max="99"
            {onkeyup}
            {disabled}
        />
        times
    {:else if block.kind === "cooldown"}
        <strong>cooldown</strong> of
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else}
        {(() => {
            const staticAssert: never = block;
            staticAssert;
        })()}
    {/if}
</div>

<style>
    div {
        border: 0.2em solid color-mix(in hsl, var(--accent), #000 25%);
        display: flex;
        gap: 0.5em;
        cursor: grab;
        padding: 0.3em;
        background-color: var(--accent);
        width: fit-content;
        align-items: center;
        color: var(--text);
    }

    input,
    select {
        font-family: "Courier New", Courier, monospace;
    }

    select {
        background-color: var(--accent);
        color: var(--text);
    }

    input[type="text"] {
        width: var(--length);
        min-width: 3ch;
    }

    input[type="number"] {
        width: 4ch;
    }
</style>
