<script lang="ts">
    import Modal from '$lib/components/Modal.svelte';
    import { buildTable } from '$lib/ts/editor-insert';

    let { onInsert, onClose }: { onInsert: (code: string) => void; onClose: () => void } = $props();

    const gridSize = 8;
    let hoveredRows = $state(3);
    let hoveredColumns = $state(3);
    let withHeader = $state(true);

    function insert(rowCount: number, columnCount: number) {
        onInsert(buildTable(rowCount, columnCount, withHeader));
    }
</script>

<Modal title="Insert table" icon="mdi:table" width="max-w-sm" onclose={onClose}>
    <div class="flex flex-col items-center gap-3">
        <div class="grid gap-1" style="grid-template-columns: repeat({gridSize}, minmax(0, 1fr));" role="grid" aria-label="Table size">
            {#each Array.from({ length: gridSize * gridSize }, (_, index) => index) as index (index)}
                {@const row = Math.floor(index / gridSize) + 1}
                {@const column = (index % gridSize) + 1}
                <button
                    type="button"
                    aria-label={`${row} by ${column} table`}
                    onmouseenter={() => { hoveredRows = row; hoveredColumns = column; }}
                    onfocus={() => { hoveredRows = row; hoveredColumns = column; }}
                    onclick={() => insert(row, column)}
                    class="h-6 w-6 rounded-sm border transition {row <= hoveredRows && column <= hoveredColumns ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'border-[var(--color-line)] bg-[var(--color-surface)]'}"
                ></button>
            {/each}
        </div>
        <p class="text-sm font-medium tabular-nums text-[var(--color-ink)]">{hoveredRows} rows x {hoveredColumns} columns</p>
        <label class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]">
            <input type="checkbox" bind:checked={withHeader} class="rounded border-[var(--color-line)]" />
            Add a header row
        </label>
    </div>
</Modal>
