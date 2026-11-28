<script lang="ts">
    import Icon from '@iconify/svelte';
    import { failureShare, formatDuration, type TimingSummary } from '#lib/ts/performance';
    const timingChart = () => import('./TimingChart.svelte');

    let { title, description, icon, summary }: {
        title: string;
        description: string;
        icon: string;
        summary: TimingSummary;
    } = $props();

    let figures = $derived([
        { label: 'Count', value: summary.count.toLocaleString() },
        { label: 'Average', value: formatDuration(summary.average_ms) },
        { label: 'Median', value: formatDuration(summary.median_ms) },
        { label: 'Slowest 5%', value: formatDuration(summary.p95_ms) },
        { label: 'Slowest', value: formatDuration(summary.max_ms) },
        { label: 'Failed', value: summary.count > 0 ? `${summary.failures.toLocaleString()} (${failureShare(summary).toFixed(1)}%)` : '-' }
    ]);
</script>

<section class="overflow-hidden rounded-md border border-[var(--color-line)] bg-[var(--color-surface)]">
    <div class="flex items-start gap-3 border-b border-[var(--color-line)] px-5 py-4">
        <Icon {icon} class="mt-0.5 text-lg text-[var(--color-accent)]" />
        <div class="min-w-0">
            <h2 class="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
            <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
        </div>
    </div>
    <dl class="grid grid-cols-3 gap-x-4 gap-y-3 px-5 py-4 sm:grid-cols-6">
        {#each figures as figure (figure.label)}
            <div class="min-w-0">
                <dt class="text-xs text-[var(--color-ink-muted)]">{figure.label}</dt>
                <dd class="truncate text-sm font-semibold tabular-nums text-[var(--color-ink)]">{figure.value}</dd>
            </div>
        {/each}
    </dl>
    <div class="px-5 pb-4">
        {#if summary.count === 0}
            <div class="flex h-44 items-center justify-center rounded-md bg-[var(--color-surface-muted)] text-sm text-[var(--color-ink-muted)]">
                Nothing recorded in this period.
            </div>
        {:else}
            {#await timingChart() then chart}
                <chart.default points={summary.points} />
            {/await}
        {/if}
    </div>
</section>
