<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { ItemSelection } from '#lib/ts/drive-selection.svelte';

    let { selection, label, class: className = '' }: { selection: ItemSelection; label: string; class?: string } = $props();
</script>

<button
    type="button"
    role="checkbox"
    aria-checked={selection.selected}
    aria-label="Select {label}"
    title="Select"
    onclick={(event) => {
        event.stopPropagation();
        selection.toggle();
    }}
    class="flex items-center justify-center rounded transition-opacity {selection.selected || selection.selecting
        ? 'opacity-100'
        : 'opacity-0 group-hover:opacity-100 focus-visible:opacity-100'} {className}"
>
    <Icon
        icon={selection.selected ? 'mdi:checkbox-marked' : 'mdi:checkbox-blank-outline'}
        class="text-xl {selection.selected ? 'text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)]'}"
    />
</button>
