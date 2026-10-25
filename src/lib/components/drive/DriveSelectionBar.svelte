<script lang="ts">
    import Icon from '@iconify/svelte';

    let { count, canMove, canStar, starred, busy, onMove, onStar, onDelete, onClear }: {
        count: number;
        canMove: boolean;
        canStar: boolean;
        starred: boolean;
        busy: boolean;
        onMove: () => void;
        onStar: () => void;
        onDelete: () => void;
        onClear: () => void;
    } = $props();

    const actionClass = 'flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-sunken)] disabled:opacity-50';
</script>

<div
    data-keep-selection
    class="fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 items-center gap-1 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1.5 shadow-2xl"
    role="toolbar"
    aria-label="Selected items"
>
    <button onclick={onClear} title="Clear selection" aria-label="Clear selection" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]">
        <Icon icon="mdi:close" class="text-lg" />
    </button>
    <span class="px-2 text-sm font-medium text-[var(--color-ink)] tabular-nums">{count} selected</span>
    <div class="mx-1 h-5 w-px bg-[var(--color-line)]"></div>
    {#if canMove}
        <button onclick={onMove} disabled={busy} class={actionClass}>
            <Icon icon="mdi:folder-move-outline" class="text-lg" /> Move
        </button>
    {/if}
    {#if canStar}
        <button onclick={onStar} disabled={busy} class={actionClass}>
            <Icon icon={starred ? 'mdi:star-off-outline' : 'mdi:star-outline'} class="text-lg" /> {starred ? 'Unstar' : 'Star'}
        </button>
    {/if}
    <button onclick={onDelete} disabled={busy} class="{actionClass} text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10">
        <Icon icon={busy ? 'mdi:loading' : 'mdi:trash-can-outline'} class="text-lg {busy ? 'animate-spin' : ''}" /> Delete
    </button>
</div>
