<script lang="ts">
    let { size, modules, label }: { size: number; modules: string; label: string } = $props();

    const QUIET_ZONE = 2;

    let darkCells = $derived(
        [...modules].flatMap((value, index) => (value === '1' ? [{ x: index % size, y: Math.floor(index / size) }] : []))
    );
</script>

<svg
    viewBox="{-QUIET_ZONE} {-QUIET_ZONE} {size + QUIET_ZONE * 2} {size + QUIET_ZONE * 2}"
    class="h-44 w-44 rounded-md bg-white"
    shape-rendering="crispEdges"
    role="img"
    aria-label={label}
>
    {#each darkCells as cell (cell.y * size + cell.x)}
        <rect x={cell.x} y={cell.y} width="1" height="1" fill="black" />
    {/each}
</svg>
