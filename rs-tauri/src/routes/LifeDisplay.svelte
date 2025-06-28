<script lang="ts">
    import type { Health } from "$lib/types";

    type Props = {
        health: Health;
    };

    let { health }: Props = $props();
</script>

<svg
    xmlns="http://www.w3.org/2000/svg"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    version="1.1"
    viewBox="0 0 1 1"
>
    <defs>
        <clipPath id="cut-off-hp">
            <rect
                x="0"
                y={(health.max_hp - health.hp) / health.max_hp}
                width="1"
                height={1 - (health.max_hp - health.hp) / health.max_hp}
            />
        </clipPath>
        <clipPath id="cut-off-es">
            <rect
                x=".5"
                y={(health.max_hp - health.hp) / health.max_hp}
                width=".5"
                height={1 - (health.max_hp - health.hp) / health.max_hp}
            />
        </clipPath>
    </defs>
    <circle
        cx=".5"
        cy=".5"
        r=".5"
        fill="var(--accent-life-bar)"
        clip-path="url(#cut-off-hp)"
    />
    <circle
        cx=".5"
        cy=".5"
        r=".5"
        fill="var(--accent-energy-bar)"
        opacity=".5"
        clip-path="url(#cut-off-es)"
    />
</svg>
