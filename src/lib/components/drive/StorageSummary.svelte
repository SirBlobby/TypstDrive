<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import { formatBytes } from '$lib/ts/settings-ui';

    type StorageStats = {
        documents_size_bytes: number;
        files_size_bytes: number;
        projects_size_bytes: number;
        total_size_bytes: number;
        quota_bytes: number;
    };

    let { collapsed }: { collapsed: boolean } = $props();

    let stats = $state<StorageStats | null>(null);
    let loadFailed = $state(false);

    let hasQuota = $derived(stats !== null && stats.quota_bytes > 0);
    let barScale = $derived(stats ? (hasQuota ? stats.quota_bytes : stats.total_size_bytes) : 0);
    let overQuota = $derived(stats !== null && hasQuota && stats.total_size_bytes >= stats.quota_bytes);
    let totalLabel = $derived.by(() => {
        if (!stats) return loadFailed ? 'Unavailable' : 'Calculating...';
        const used = formatBytes(stats.total_size_bytes);
        return hasQuota ? `${used} / ${formatBytes(stats.quota_bytes)}` : used;
    });
    let breakdownLabel = $derived(
        stats
            ? `Documents ${formatBytes(stats.documents_size_bytes)} · Files ${formatBytes(stats.files_size_bytes)} · Projects ${formatBytes(stats.projects_size_bytes)}`
            : undefined
    );

    function share(bytes: number) {
        return barScale > 0 ? Math.min(100, (bytes / barScale) * 100) : 0;
    }

    function openStorageSettings() {
        goto('/settings?section=storage');
    }

    onMount(async () => {
        try {
            const res = await fetch('/api/auth/storage');
            if (res.ok) stats = await res.json();
            else loadFailed = true;
        } catch {
            loadFailed = true;
        }
    });
</script>

{#if collapsed}
    <button
        onclick={openStorageSettings}
        title="Storage: {totalLabel}"
        aria-label="Storage: {totalLabel}"
        class="flex w-full justify-center rounded-md py-1.5 hover:bg-[var(--color-surface-muted)] transition-colors {overQuota ? 'text-[var(--color-danger)]' : 'text-[var(--color-ink-muted)] hover:text-[var(--color-accent)]'}"
    >
        <Icon icon="mdi:database-outline" class="text-lg" />
    </button>
{:else}
    <button
        onclick={openStorageSettings}
        aria-label="Storage: {totalLabel}. Open storage settings"
        title={breakdownLabel}
        class="w-full rounded-md px-2 py-1.5 text-left hover:bg-[var(--color-surface-muted)] transition-colors"
    >
        <div class="flex items-center gap-1.5 text-xs">
            <Icon icon="mdi:database-outline" class="text-sm text-[var(--color-accent)]" />
            <span class="flex-1 font-medium text-[var(--color-ink)]">Storage</span>
            <span class="tabular-nums {overQuota ? 'font-medium text-[var(--color-danger)]' : 'text-[var(--color-ink-muted)]'}">{totalLabel}</span>
        </div>

        {#if stats}
            <div class="mt-1.5 flex h-1 w-full overflow-hidden rounded-full bg-[var(--color-surface-sunken)]" role="img" aria-label="Storage breakdown">
                {#if overQuota}
                    <div class="h-full w-full bg-[var(--color-danger)]"></div>
                {:else}
                    <div class="h-full bg-[var(--color-accent)]" style="width: {share(stats.documents_size_bytes)}%"></div>
                    <div class="h-full bg-purple-500" style="width: {share(stats.files_size_bytes)}%"></div>
                    <div class="h-full bg-teal-500" style="width: {share(stats.projects_size_bytes)}%"></div>
                {/if}
            </div>
        {:else if !loadFailed}
            <div class="mt-1.5 h-1 w-full animate-pulse rounded-full bg-[var(--color-surface-sunken)]"></div>
        {/if}
    </button>
{/if}
