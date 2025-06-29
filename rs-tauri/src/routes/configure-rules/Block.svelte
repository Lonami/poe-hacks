<script lang="ts">
    import type { BlockDefinition } from "$lib/types";
    import {
        BLOCK_CLICK_VARIABLES,
        BLOCK_SCROLL_VARIABLES,
        BLOCK_STAT_CONDITIONS,
        BLOCK_STAT_VARIABLES,
    } from "$lib/constants";
    import { serializeBlockDefinition } from "$lib/serde";

    import type {
        DragEventHandler,
        KeyboardEventHandler,
        MouseEventHandler,
    } from "svelte/elements";
    import GripAreaDisplay from "./GripAreaDisplay.svelte";
    import ContextMenu from "./ContextMenu.svelte";

    type Props = {
        block: BlockDefinition;
    } & (
        | {
              onBlockMoved: () => void;
              onBlockDeleted: () => void;
              isReadonly?: undefined;
          }
        | {
              onBlockMoved?: undefined;
              onBlockDeleted?: undefined;
              isReadonly: true;
          }
    );

    const {
        block = $bindable(),
        onBlockMoved,
        onBlockDeleted,
        isReadonly: disabled,
    }: Props = $props();

    let contextMenu: ContextMenu;

    const ondragstart: DragEventHandler<HTMLDivElement> = (event) => {
        const dt = event.dataTransfer!;
        dt.setData("text/x-poehacks-block", serializeBlockDefinition(block));
        dt.effectAllowed = disabled ? "copy" : "copyMove";
        onBlockMoved?.();
    };

    const oncontextmenu: MouseEventHandler<HTMLDivElement> = (event) => {
        event.preventDefault();
        contextMenu.openAt(event.x, event.y);
    };

    let accent = $derived.by(() => {
        switch (block.kind) {
            case "stat":
            case "key":
            case "mouse":
                return "condition";
            case "press":
            case "type":
            case "disconnect":
            case "click":
            case "scroll":
                return "action";
            case "cooldown":
            case "delay":
                return "timing";
            default:
                const assertNever: never = block;
                throw assertNever;
        }
    });

    const keyFromKeyboardEvent = (e: KeyboardEvent): string => {
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
        if (/.|F\d+/.test(e.key)) {
            value += e.key === " " ? "Spc" : e.key.toUpperCase();
        }
        return value;
    };

    const onkeyup: KeyboardEventHandler<HTMLInputElement> = (event) => {
        if ("value" in block) {
            block.value = event.currentTarget.value;
        }
    };
</script>

<div
    draggable="true"
    {ondragstart}
    oncontextmenu={disabled ? undefined : oncontextmenu}
    role="row"
    tabindex="-1"
    style="--accent: var(--accent-{accent}); --text: var(--accent-{accent}-text); --length: {'value' in
    block
        ? block.value.length + 2
        : 0}ch;"
>
    <GripAreaDisplay height={"20"} />
    {#if block.kind === "stat"}
        <strong>stat</strong>
        <select value={block.variable} {disabled}>
            {#each BLOCK_STAT_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <select value={block.condition} {disabled}>
            {#each BLOCK_STAT_CONDITIONS as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else if block.kind === "key"}
        <strong>key</strong>
        <input
            type="text"
            value={block.value}
            maxlength="1"
            onkeydown={(e) => {
                e.preventDefault();
                block.value = keyFromKeyboardEvent(e);
            }}
            {disabled}
        />
        pressed
    {:else if block.kind === "mouse"}
        <strong>mouse</strong>
        <select value={block.variable} {disabled}>
            {#each BLOCK_CLICK_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
        clicked
    {:else if block.kind === "press"}
        <strong>press</strong>
        <input
            type="text"
            value={block.value}
            maxlength="1"
            onkeydown={(e) => {
                e.preventDefault();
                block.value = keyFromKeyboardEvent(e);
            }}
            {disabled}
        />
    {:else if block.kind === "type"}
        <strong>type</strong>
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else if block.kind === "disconnect"}
        <strong>disconnect</strong>
    {:else if block.kind === "click"}
        <strong>click</strong>
        <select value={block.variable} {disabled}>
            {#each BLOCK_CLICK_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
    {:else if block.kind === "scroll"}
        <strong>scroll</strong>
        <select value={block.variable} {disabled}>
            {#each BLOCK_SCROLL_VARIABLES as value}
                <option {value}>{value}</option>
            {/each}
        </select>
    {:else if block.kind === "cooldown"}
        <strong>cooldown</strong> of
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else if block.kind === "delay"}
        <strong>delay</strong> of
        <input type="text" value={block.value} {onkeyup} {disabled} />
    {:else}
        {(() => {
            const staticAssert: never = block;
            staticAssert;
        })()}
    {/if}
</div>
<ContextMenu bind:this={contextMenu}>
    <button
        onclick={(e) => {
            e.preventDefault();
            contextMenu.close();
            onBlockDeleted?.();
        }}
    >
        Delete
    </button>
</ContextMenu>

<style>
    div {
        border: 0.2em solid color-mix(in hsl, var(--accent), #000 25%);
        display: flex;
        gap: 1ch;
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
</style>
