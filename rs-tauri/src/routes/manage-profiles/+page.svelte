<script lang="ts">
    const profiles = ["default", "SSF", "HC VFoS"];
</script>

<ul>
    {#each profiles as profile, i}
        <li>
            <input type="checkbox" checked={!i} disabled />
            <input type="text" value={profile} disabled={!i} /> ({i * 3} rules)
            <button>Duplicate</button>
            <button class="danger" disabled={!i}>Delete</button>
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
