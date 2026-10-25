<script lang="ts">
    import Icon from '@iconify/svelte';
    import { exportFormats, exportSavedItem, type ExportFormat } from '#lib/ts/typst-api';

    let { kind, id, name, onstart }: {
        kind: 'document' | 'project';
        id: string;
        name: string;
        onstart: () => void;
    } = $props();

    function exportAs(event: MouseEvent, format: ExportFormat) {
        event.stopPropagation();
        onstart();
        exportSavedItem(kind, id, format, name).catch(() => {
            alert(`Could not export ${name} as ${format.toUpperCase()}. Open it to see any errors.`);
        });
    }
</script>

<div class="h-px bg-[var(--color-line)] my-1"></div>
<p class="px-4 pt-1 pb-0.5 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Export</p>
{#each exportFormats as option (option.format)}
    <button
        onclick={(event) => exportAs(event, option.format)}
        class="w-full text-left px-4 py-1.5 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2"
    >
        <Icon icon={option.icon} class="text-lg {option.color}" />
        {option.label}
    </button>
{/each}
