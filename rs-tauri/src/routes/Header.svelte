<script lang="ts">
    import { ROUTES } from "$lib/constants";
    import { G } from "$lib/globals.svelte";
    type Props = {
        pathname: string;
    };

    const { pathname }: Props = $props();
</script>

<header>
    <nav>
        <ul>
            {#each Object.entries(ROUTES) as [href, text]}
                <li>
                    <a {href} class:active={href === pathname}>{text}</a>
                </li>
            {/each}
        </ul>
    </nav>

    <div
        title={G.profilesSynced ? "Profiles saved" : "Profiles failed to save"}
    >
        <svg
            xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            version="1.1"
            viewBox="0 0 1 1"
            style="--status: var(--status-{G.profilesSynced ? 'ok' : 'err'})"
            height={"16px"}
        >
            <circle cx=".5" cy=".5" r=".5" />
        </svg>
    </div>

    <label>
        Profile:
        <select
            value={G.profiles.find((p) => p.active)?.name}
            onchange={(e) => {
                for (const p of G.profiles) {
                    p.active = p.name === e.currentTarget.value;
                }
            }}
        >
            {#each G.profiles as profile}
                <option value={profile.name}>{profile.name}</option>
            {/each}
        </select>
    </label>
</header>

<style>
    header {
        display: flex;
        align-items: center;
        box-shadow: 0 0 2px #000;
        padding-right: 0.5em;
        gap: 1em;
    }
    nav {
        flex-grow: 1;
    }
    ul {
        list-style-type: none;
        margin: 0;
        padding: 0;
        display: flex;
    }
    li {
        display: flex;
    }
    div {
        display: flex;
    }
    circle {
        fill: var(--status);
    }
    a {
        text-decoration: none;
        color: #000;
    }
</style>
