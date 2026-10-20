<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatBytes, secondaryButtonClass } from '$lib/ts/settings-ui';
    import { fetchServerInfo, type ServerInfo } from '$lib/ts/server-info';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';
    import EnvironmentVariables from './EnvironmentVariables.svelte';

    let info = $state<ServerInfo | null>(null);
    let loading = $state(true);
    let loadError = $state('');

    let realtimeStatus = $derived.by(() => {
        if (!info?.realtime.redis_enabled) return 'Single instance';
        return info.realtime.connected ? 'Redis connected' : 'Redis not connected';
    });

    async function load() {
        loading = true;
        loadError = '';
        try {
            info = await fetchServerInfo();
        } catch (err) {
            loadError = (err as Error).message;
        }
        loading = false;
    }

    onMount(load);
</script>

<SettingsHeader title="Server" description="How this instance is set up: its database, real-time coordination, file storage and environment." icon="mdi:server-outline">
    {#snippet actions()}
        <button onclick={load} disabled={loading} class={secondaryButtonClass}>
            <Icon icon="mdi:refresh" class="text-base {loading ? 'animate-spin' : ''}" />
            Refresh
        </button>
    {/snippet}
</SettingsHeader>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !info}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Database" description="Set with DATABASE_URL and DB_TYPE.">
            <SettingsRow label="Type"><span class="text-sm text-[var(--color-ink)]">{info.database.kind}</span></SettingsRow>
            <SettingsRow label="Server version">
                <span class="truncate text-sm text-[var(--color-ink)]" title={info.database.version ?? ''}>{info.database.version ?? 'Unknown'}</span>
            </SettingsRow>
            <SettingsRow label="Schema version"><span class="text-sm tabular-nums text-[var(--color-ink)]">{info.database.schema_version}</span></SettingsRow>
            <SettingsRow label="Connection pool"><span class="text-sm tabular-nums text-[var(--color-ink)]">{info.database.max_connections} connections</span></SettingsRow>
        </SettingsCard>

        <SettingsCard title="Real-time collaboration" description="With REDIS_URL set, edits and access changes are shared between every app instance.">
            <SettingsRow label="Coordination">
                <span class="flex items-center gap-1.5 text-sm text-[var(--color-ink)]">
                    {#if info.realtime.redis_enabled}
                        <span class="h-2 w-2 rounded-full {info.realtime.connected ? 'bg-[var(--color-success)]' : 'bg-[var(--color-danger)]'}"></span>
                    {/if}
                    {realtimeStatus}
                </span>
            </SettingsRow>
            <SettingsRow label="This instance"><code class="text-xs font-mono text-[var(--color-ink)]">{info.realtime.instance_id}</code></SettingsRow>
        </SettingsCard>

        <SettingsCard title="File storage" description="Uploaded fonts, images and other files. Set S3_BUCKET to keep them in S3-compatible object storage.">
            <SettingsRow label="Backend">
                <span class="text-sm text-[var(--color-ink)]">{info.file_storage.object_storage_enabled ? 'S3-compatible object storage' : 'Database'}</span>
            </SettingsRow>
            <SettingsRow label="Files in the database"><span class="text-sm tabular-nums text-[var(--color-ink)]">{info.file_storage.files_in_database}</span></SettingsRow>
            <SettingsRow label="Files in object storage"><span class="text-sm tabular-nums text-[var(--color-ink)]">{info.file_storage.files_in_object_storage}</span></SettingsRow>
            {#if info.file_storage.object_storage_enabled}
                <SettingsRow label="Memory cache"><span class="text-sm tabular-nums text-[var(--color-ink)]">{formatBytes(info.file_storage.cache_limit_bytes)}</span></SettingsRow>
            {/if}
        </SettingsCard>

        <EnvironmentVariables variables={info.environment} />

        <p class="text-xs text-[var(--color-ink-muted)]">TypstDrive {info.server_version}</p>
    </div>
{/if}
