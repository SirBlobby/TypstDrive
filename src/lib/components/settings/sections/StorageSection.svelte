<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatBytes } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';

    type StorageStats = {
        documents_size_bytes: number;
        files_size_bytes: number;
        projects_size_bytes: number;
        total_size_bytes: number;
        quota_bytes: number;
    };

    let stats = $state<StorageStats | null>(null);
    let loadFailed = $state(false);

    let hasQuota = $derived(stats !== null && stats.quota_bytes > 0);
    let barScale = $derived(stats ? (hasQuota ? stats.quota_bytes : stats.total_size_bytes) : 0);

    function share(bytes: number) {
        return barScale > 0 ? Math.min(100, (bytes / barScale) * 100) : 0;
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

<SettingsHeader title="Storage" description="How much space your documents, uploaded files and projects use." icon="mdi:harddisk" />

<SettingsCard padded>
    {#if loadFailed}
        <p class="text-sm text-[var(--color-danger)]">Could not load storage usage.</p>
    {:else if !stats}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Calculating...</p>
    {:else}
        <div class="flex items-end justify-between gap-4">
            <div>
                <p class="text-xs font-medium uppercase tracking-wider text-[var(--color-ink-muted)]">Total used</p>
                <p class="mt-1 text-3xl font-semibold text-[var(--color-ink)] tabular-nums">{formatBytes(stats.total_size_bytes)}</p>
            </div>
            <p class="text-sm text-[var(--color-ink-muted)] tabular-nums">
                {hasQuota ? `of ${formatBytes(stats.quota_bytes)}` : 'No storage limit'}
            </p>
        </div>

        <div class="mt-5 flex h-2.5 w-full overflow-hidden rounded-sm bg-[var(--color-surface-sunken)]" role="img" aria-label="Storage breakdown">
            <div class="h-full bg-[var(--color-accent)]" style="width: {share(stats.documents_size_bytes)}%"></div>
            <div class="h-full bg-purple-500" style="width: {share(stats.files_size_bytes)}%"></div>
            <div class="h-full bg-teal-500" style="width: {share(stats.projects_size_bytes)}%"></div>
        </div>

        <dl class="mt-5 grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div class="flex items-center gap-3 rounded-md border border-[var(--color-line)] p-3">
                <span class="h-3 w-3 flex-shrink-0 rounded-full bg-[var(--color-accent)]"></span>
                <div class="min-w-0">
                    <dt class="text-xs text-[var(--color-ink-muted)]">Documents</dt>
                    <dd class="text-sm font-semibold text-[var(--color-ink)] tabular-nums">{formatBytes(stats.documents_size_bytes)}</dd>
                </div>
            </div>
            <div class="flex items-center gap-3 rounded-md border border-[var(--color-line)] p-3">
                <span class="h-3 w-3 flex-shrink-0 rounded-full bg-purple-500"></span>
                <div class="min-w-0">
                    <dt class="text-xs text-[var(--color-ink-muted)]">Images, fonts and other files</dt>
                    <dd class="text-sm font-semibold text-[var(--color-ink)] tabular-nums">{formatBytes(stats.files_size_bytes)}</dd>
                </div>
            </div>
            <div class="flex items-center gap-3 rounded-md border border-[var(--color-line)] p-3">
                <span class="h-3 w-3 flex-shrink-0 rounded-full bg-teal-500"></span>
                <div class="min-w-0">
                    <dt class="text-xs text-[var(--color-ink-muted)]">Projects</dt>
                    <dd class="text-sm font-semibold text-[var(--color-ink)] tabular-nums">{formatBytes(stats.projects_size_bytes)}</dd>
                </div>
            </div>
        </dl>
    {/if}
</SettingsCard>
