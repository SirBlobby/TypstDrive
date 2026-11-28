<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';

    let { label, value, detail, icon, share }: {
        label: string;
        value: string;
        detail?: string;
        icon: string;
        share?: number | null;
    } = $props();

    let barColor = $derived(
        share == null ? '' : share >= 90 ? 'bg-[var(--color-danger)]' : share >= 75 ? 'bg-amber-500' : 'bg-[var(--color-accent)]'
    );
</script>

<div class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-3">
    <p class="flex items-center gap-1.5 text-xs font-medium text-[var(--color-ink-muted)]">
        <Icon {icon} class="text-sm" />
        {label}
    </p>
    <p class="mt-1 text-xl font-semibold tabular-nums text-[var(--color-ink)]">{value}</p>
    {#if share != null}
        <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-[var(--color-surface-sunken)]">
            <div class="h-full rounded-full {barColor}" style="width: {Math.min(100, Math.max(0, share))}%"></div>
        </div>
    {/if}
    {#if detail}
        <p class="mt-1 truncate text-xs text-[var(--color-ink-muted)]" title={detail}>{detail}</p>
    {/if}
</div>
