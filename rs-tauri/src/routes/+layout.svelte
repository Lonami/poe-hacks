<script lang="ts">
    import type { Snippet } from "svelte";
    import { ROUTES } from "$lib/constants";
    import { fly } from "svelte/transition";
    import Header from "./Header.svelte";
    import { flyIn } from "$lib/transition";
    import { page } from "$app/state";

    type Props = {
        children: Snippet;
    };

    const { children }: Props = $props();

    let prev = $state(location.href);

    const routes = Object.keys(ROUTES);
    const x = $derived(
        50 *
            Math.sign(routes.indexOf(page.url.pathname) - routes.indexOf(prev)),
    );

    const carousselOut: typeof fly = (node, props) =>
        fly(node, { ...props, x: -x });

    const carousselIn: typeof flyIn = (node, props) =>
        flyIn(node, { ...props, x });
</script>

<Header pathname={page.url.pathname} />

{#key page.url.pathname}
    <main
        out:carousselOut={{ duration: 100 }}
        in:carousselIn={{ duration: 100, delay: 100 }}
        onintroend={() => {
            prev = page.url.pathname;
        }}
    >
        {@render children()}
    </main>
{/key}

<style>
    :root {
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 24px;
        font-weight: 400;

        color: #0f0f0f;
        background-color: #f6f6f6;
        scrollbar-color: #0f0f0f #f6f6f6;

        font-synthesis: none;
        text-rendering: optimizeLegibility;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        -webkit-text-size-adjust: 100%;
    }

    main {
        display: flex;
        flex-direction: column;
        flex-grow: 1;
        overflow: hidden;
    }
</style>
