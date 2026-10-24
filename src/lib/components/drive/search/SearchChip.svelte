<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { SearchToken } from '$lib/ts/drive-search';

    let { token, onremove }: { token: SearchToken; onremove: () => void } = $props();
</script>

<span class="group/chip inline-flex items-center gap-1 shrink-0 max-w-56 rounded-md bg-[var(--color-surface-sunken)] border border-[var(--color-line)] pl-2 pr-1 py-0.5 text-xs">
    <span class="font-semibold text-[var(--color-ink)]">{token.key}:</span>
    <span class="truncate text-[var(--color-ink-muted)]">{token.value}</span>
    <button
        aria-label={`Remove ${token.key} filter`}
        onmousedown={(event) => event.preventDefault()}
        onclick={(event) => { event.stopPropagation(); onremove(); }}
        class="rounded p-0.5 text-[var(--color-ink-muted)] opacity-60 group-hover/chip:opacity-100 hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-muted)]"
    >
        <Icon icon="mdi:close" class="text-xs" />
    </button>
</span>
