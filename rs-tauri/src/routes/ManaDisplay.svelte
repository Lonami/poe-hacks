<script lang="ts">
    import type { Mana } from "$lib/types";

    type Props = {
        mana: Mana;
    };

    let { mana }: Props = $props();
</script>

<svg
    xmlns="http://www.w3.org/2000/svg"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    version="1.1"
    viewBox="0 0 1 1"
>
    <defs>
        <clipPath id="cut-off-mana">
            <rect
                x="0"
                y={(mana.max_mana - mana.mana) / mana.max_mana}
                width="1"
                height={1 - (mana.max_mana - mana.mana) / mana.max_mana}
            />
        </clipPath>
    </defs>
    <circle
        cx=".5"
        cy=".5"
        r=".5"
        fill="var(--accent-mana-bar)"
        clip-path="url(#cut-off-mana)"
    />
</svg>
