<script lang="ts">
    import type { DiffLine } from '$lib/ts/line-diff';

    let { lines, showUnchanged }: { lines: DiffLine[]; showUnchanged: boolean } = $props();

    const CONTEXT_LINES = 3;

    type Row = { type: 'line'; line: DiffLine } | { type: 'gap'; hidden: number };

    let rows = $derived.by<Row[]>(() => {
        if (showUnchanged) return lines.map((line) => ({ type: 'line', line }));
        const nearChange = lines.map(() => false);
        lines.forEach((line, index) => {
            if (line.kind === 'same') return;
            for (let near = Math.max(0, index - CONTEXT_LINES); near <= Math.min(lines.length - 1, index + CONTEXT_LINES); near++) {
                nearChange[near] = true;
            }
        });
        const result: Row[] = [];
        let hidden = 0;
        lines.forEach((line, index) => {
            if (nearChange[index]) {
                if (hidden > 0) result.push({ type: 'gap', hidden });
                hidden = 0;
                result.push({ type: 'line', line });
            } else {
                hidden++;
            }
        });
        if (hidden > 0) result.push({ type: 'gap', hidden });
        return result;
    });

    const rowStyles = {
        same: 'text-[var(--color-ink)]',
        added: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300',
        removed: 'bg-rose-500/10 text-rose-700 dark:text-rose-300 line-through decoration-rose-400/60'
    };

    const markers = { same: ' ', added: '+', removed: '-' };
</script>

<div class="overflow-x-auto rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] font-mono text-[13px] leading-6">
    {#each rows as row, index (index)}
        {#if row.type === 'gap'}
            <div class="border-y border-dashed border-[var(--color-line)] bg-[var(--color-surface-muted)] px-4 py-1 text-xs text-[var(--color-ink-muted)]">
                {row.hidden} unchanged {row.hidden === 1 ? 'line' : 'lines'}
            </div>
        {:else}
            <div class="flex {rowStyles[row.line.kind]}">
                <span class="w-12 flex-shrink-0 select-none pr-2 text-right text-[var(--color-ink-muted)] opacity-60">{row.line.oldNumber ?? ''}</span>
                <span class="w-12 flex-shrink-0 select-none pr-2 text-right text-[var(--color-ink-muted)] opacity-60">{row.line.newNumber ?? ''}</span>
                <span class="w-5 flex-shrink-0 select-none text-center">{markers[row.line.kind]}</span>
                <span class="whitespace-pre-wrap break-all pr-4">{row.line.text || ' '}</span>
            </div>
        {/if}
    {:else}
        <p class="p-6 text-center text-sm text-[var(--color-ink-muted)]">This version is empty.</p>
    {/each}
</div>
