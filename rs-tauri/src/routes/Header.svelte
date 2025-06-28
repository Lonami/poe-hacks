<script lang="ts">
    import { ROUTES } from "$lib/constants";
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
    <label>
        Profile:
        <select>
            <option>default</option>
        </select>
    </label>
</header>

<style>
    header {
        display: flex;
        align-items: center;
        box-shadow: 0 0 2px #000;
        margin-bottom: 1px;
        padding-right: 0.5em;
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
    a {
        padding: 0.25em 0.5em;
        text-decoration: none;
        color: #000;
        box-shadow: inset -1px -1px 2px #000;
        transition: box-shadow 200ms;
    }
    .active {
        border: none;
        box-shadow: inset 1px 1px 2px #000;
    }
</style>
