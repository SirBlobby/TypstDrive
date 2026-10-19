<script lang="ts">
    import Icon from '@iconify/svelte';
    import { sortOptions, type DriveFilters, type ViewMode } from '$lib/ts/drive-filters';

    let { filters = $bindable(), viewMode = $bindable() }: {
        filters: DriveFilters;
        viewMode: ViewMode;
    } = $props();

    function toggleSortDirection() {
        filters = { ...filters, sortDirection: filters.sortDirection === 'asc' ? 'desc' : 'asc' };
    }
</script>

{#snippet filterSelect(label: string, icon: string, value: string, options: { value: string; label: string }[], onchange: (value: string) => void)}
    <label class="relative inline-flex items-center gap-1.5 pl-3 pr-8 py-1.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] text-sm text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] cursor-pointer transition-colors">
        <Icon {icon} class="text-base flex-shrink-0" />
        <span class="sr-only">{label}</span>
        <select
            {value}
            onchange={(e) => onchange((e.currentTarget as HTMLSelectElement).value)}
            aria-label={label}
            class="appearance-none bg-transparent outline-none cursor-pointer font-medium"
        >
            {#each options as option}
                <option value={option.value} class="text-[var(--color-ink)] bg-[var(--color-surface)]">{option.label}</option>
            {/each}
        </select>
        <Icon icon="mdi:chevron-down" class="absolute right-2.5 text-base pointer-events-none" />
    </label>
{/snippet}

<div class="flex items-center justify-end gap-2 flex-wrap mb-6">
    <div class="flex items-center gap-2">
        {@render filterSelect('Sort by', 'mdi:sort', filters.sortField, sortOptions, (value) => (filters = { ...filters, sortField: value as DriveFilters['sortField'] }))}
        <button
            onclick={toggleSortDirection}
            title={filters.sortDirection === 'asc' ? 'Ascending' : 'Descending'}
            aria-label={filters.sortDirection === 'asc' ? 'Sort ascending' : 'Sort descending'}
            class="p-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
        >
            <Icon icon={filters.sortDirection === 'asc' ? 'mdi:sort-ascending' : 'mdi:sort-descending'} class="text-base" />
        </button>

        <div class="flex rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-0.5" role="radiogroup" aria-label="Layout">
            {#each [{ value: 'grid', icon: 'mdi:view-grid-outline', label: 'Grid view' }, { value: 'list', icon: 'mdi:view-list-outline', label: 'List view' }] as option}
                <button
                    role="radio"
                    aria-checked={viewMode === option.value}
                    aria-label={option.label}
                    title={option.label}
                    onclick={() => (viewMode = option.value as ViewMode)}
                    class="p-1.5 rounded-md transition-colors {viewMode === option.value
                        ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                        : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    <Icon icon={option.icon} class="text-base" />
                </button>
            {/each}
        </div>
    </div>
</div>
