<script lang="ts">
    type Props = {
        height: string;
    };

    const { height }: Props = $props();
</script>

<svg
    xmlns="http://www.w3.org/2000/svg"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    version="1.1"
    viewBox="0 0 1 1"
    {height}
>
    <circle cx=".25" cy=".1" r=".075" />
    <circle cx=".50" cy=".1" r=".075" />
    <circle cx=".75" cy=".1" r=".075" />

    <circle cx=".25" cy=".35" r=".075" />
    <circle cx=".50" cy=".35" r=".075" />
    <circle cx=".75" cy=".35" r=".075" />

    <circle cx=".25" cy=".6" r=".075" />
    <circle cx=".50" cy=".6" r=".075" />
    <circle cx=".75" cy=".6" r=".075" />

    <circle cx=".25" cy=".85" r=".075" />
    <circle cx=".50" cy=".85" r=".075" />
    <circle cx=".75" cy=".85" r=".075" />
</svg>

<style>
    circle {
        fill: var(--accent-grip-area);
    }
</style>
