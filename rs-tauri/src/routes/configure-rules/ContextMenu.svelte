<script lang="ts">
    import type { Snippet } from "svelte";
    import type { EventHandler } from "svelte/elements";

    let dialog: HTMLDialogElement;

    type Props = {
        onClose?: EventHandler<Event, HTMLDialogElement>;
        children: Snippet;
    };

    const { onClose, children }: Props = $props();

    let style = $state<string>();

    export const openAt = (x: number, y: number) => {
        style = `--x: ${x}px; --y: ${y}px`;
        dialog.showModal();
    };

    export const close = (returnValue?: string) => {
        dialog.close(returnValue);
    };
</script>

<dialog bind:this={dialog} onclose={onClose} closedby="any" {style}>
    <form method="dialog">
        {@render children()}
    </form>
</dialog>

<style>
    dialog {
        position: absolute;
        margin: 0;
        padding: 0;
        top: var(--y);
        left: var(--x);
    }
</style>
