<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatBytes } from '#lib/ts/settings-ui';
    import { fetchPerformance, formatUptime, TIMING_DETAILS, type PerformancePeriod, type PerformanceReport } from '#lib/ts/performance';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import StatTile from './performance/StatTile.svelte';
    import TimingCard from './performance/TimingCard.svelte';

    const REFRESH_MS = 15000;
    const periods: { value: PerformancePeriod; label: string; title: string }[] = [
        { value: '1h', label: '1H', title: 'Last hour' },
        { value: '6h', label: '6H', title: 'Last 6 hours' },
        { value: '24h', label: '24H', title: 'Last 24 hours' }
    ];

    let period = $state<PerformancePeriod>('1h');
    let report = $state<PerformanceReport | null>(null);
    let loadError = $state('');
    let refreshing = $state(false);
    let updatedAt = $state<Date | null>(null);

    function share(used: number | null, total: number | null) {
        return used != null && total ? (used / total) * 100 : null;
    }

    function bytesOrDash(bytes: number | null) {
        return bytes == null ? '-' : formatBytes(bytes);
    }

    let tiles = $derived.by(() => {
        if (!report) return [];
        const { system, activity, content } = report;
        const memoryUsed = system.memory_total_bytes != null && system.memory_available_bytes != null ? system.memory_total_bytes - system.memory_available_bytes : null;
        const diskUsed = system.disk_total_bytes != null && system.disk_free_bytes != null ? system.disk_total_bytes - system.disk_free_bytes : null;
        const load = system.load_average;
        return [
            { label: 'Uptime', icon: 'mdi:timer-outline', value: formatUptime(report.uptime_seconds), detail: `Instance ${report.instance_id}` },
            {
                label: 'Server CPU',
                icon: 'mdi:cpu-64-bit',
                value: system.process_cpu_percent == null ? '-' : `${system.process_cpu_percent.toFixed(1)}%`,
                detail: `${system.cpu_cores} cores${system.process_threads ? `, ${system.process_threads} threads` : ''}`,
                share: system.process_cpu_percent == null ? null : system.process_cpu_percent / system.cpu_cores
            },
            {
                label: 'Load average',
                icon: 'mdi:chart-line-variant',
                value: load ? load[0].toFixed(2) : '-',
                detail: load ? `5 min ${load[1].toFixed(2)}, 15 min ${load[2].toFixed(2)}` : 'Not available',
                share: load ? (load[0] / system.cpu_cores) * 100 : null
            },
            {
                label: 'Server memory',
                icon: 'mdi:memory',
                value: bytesOrDash(system.process_memory_bytes),
                detail: system.memory_total_bytes ? `Machine: ${bytesOrDash(memoryUsed)} of ${formatBytes(system.memory_total_bytes)} used` : undefined,
                share: share(memoryUsed, system.memory_total_bytes)
            },
            {
                label: 'Disk',
                icon: 'mdi:harddisk',
                value: `${bytesOrDash(system.disk_free_bytes)} free`,
                detail: `${bytesOrDash(system.disk_total_bytes)} total at ${system.disk_path}`,
                share: share(diskUsed, system.disk_total_bytes)
            },
            {
                label: 'Live editing',
                icon: 'mdi:account-multiple-outline',
                value: `${activity.live_connections}`,
                detail: `${activity.live_connections === 1 ? 'connection' : 'connections'} in ${activity.live_documents} open ${activity.live_documents === 1 ? 'file' : 'files'}`
            },
            {
                label: 'LaTeX compiles running',
                icon: 'mdi:sigma',
                value: `${activity.latex_compiles_running} of ${activity.latex_compiles_allowed}`,
                share: (activity.latex_compiles_running / activity.latex_compiles_allowed) * 100
            },
            {
                label: 'Content',
                icon: 'mdi:folder-multiple-outline',
                value: `${content.documents.toLocaleString()} docs`,
                detail: `${content.users.toLocaleString()} users, ${content.projects.toLocaleString()} projects, ${content.files.toLocaleString()} files (${formatBytes(content.file_bytes)})`
            }
        ];
    });

    async function load() {
        refreshing = true;
        try {
            report = await fetchPerformance(period);
            updatedAt = new Date();
            loadError = '';
        } catch (err) {
            loadError = (err as Error).message;
        }
        refreshing = false;
    }

    $effect(() => {
        period;
        load();
    });

    onMount(() => {
        const timer = setInterval(() => {
            if (document.visibilityState === 'visible') load();
        }, REFRESH_MS);
        return () => clearInterval(timer);
    });
</script>

<SettingsHeader title="Performance" description="Server load, page load times and how long Typst and LaTeX compiles take." icon="mdi:speedometer">
    {#snippet actions()}
        <div class="flex rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-0.5 text-xs font-semibold" role="radiogroup" aria-label="Period">
            {#each periods as item (item.value)}
                <button
                    role="radio"
                    aria-checked={period === item.value}
                    title={item.title}
                    onclick={() => (period = item.value)}
                    class="rounded-md px-2.5 py-1 transition {period === item.value
                        ? 'bg-[var(--color-surface)] text-[var(--color-accent)] shadow-sm'
                        : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    {item.label}
                </button>
            {/each}
        </div>
        <button
            onclick={load}
            disabled={refreshing}
            title="Refresh"
            aria-label="Refresh"
            class="rounded-md border border-[var(--color-line)] p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] disabled:opacity-60"
        >
            <Icon icon="mdi:refresh" class="text-base {refreshing ? 'animate-spin' : ''}" />
        </button>
    {/snippet}
</SettingsHeader>

{#if loadError && !report}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !report}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {#each tiles as tile (tile.label)}
                <StatTile {...tile} />
            {/each}
        </div>

        {#each TIMING_DETAILS as timing (timing.key)}
            <TimingCard title={timing.title} description={timing.description} icon={timing.icon} summary={report.timings[timing.key]} />
        {/each}

        <p class="text-xs leading-relaxed text-[var(--color-ink-muted)]">
            {#if loadError}<span class="text-[var(--color-danger)]">{loadError} </span>{/if}
            Times are kept in memory for the last 24 hours and start over when the server restarts. With several app instances, this page shows the instance that answered.
            {#if updatedAt}Updated {updatedAt.toLocaleTimeString()}, refreshes every 15 seconds.{/if}
        </p>
    </div>
{/if}
