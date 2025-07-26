<script lang="ts">
    import { G } from "$lib/globals.svelte";
</script>

<ul>
    {#each G.profiles as profile, i}
        <li>
            <input type="checkbox" checked={profile.active} disabled />
            <input type="text" bind:value={profile.name} /> ({profile.rules
                .length} rule{[profile.rules.length === 1 ? "" : "s"]})
            <button
                onclick={() => {
                    G.profiles.splice(i + 1, 0, {
                        ...structuredClone($state.snapshot(profile)),
                        active: false,
                    });
                }}>Duplicate</button
            >
            <button
                class="danger"
                disabled={G.profiles.length === 1}
                onclick={() => G.profiles.splice(i, 1)}>Delete</button
            >
        </li>
    {/each}
</ul>

<style>
    ul {
        list-style-type: none;
        margin: 1em;
        padding: 0;
    }
    li {
        display: flex;
        align-items: center;
        gap: 1em;
    }
</style>
