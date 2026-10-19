<script lang="ts">
    import Icon from '@iconify/svelte';
    import { Chart, LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip } from 'chart.js';

    Chart.register(LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip);

    type UsagePoint = { date: string; count: number };
    type UsagePeriod = '1hr' | '1day' | '1week';

    const periods: { value: UsagePeriod; label: string; title: string }[] = [
        { value: '1hr', label: '1H', title: 'Last 60 minutes' },
        { value: '1day', label: '24H', title: 'Last 24 hours' },
        { value: '1week', label: '7D', title: 'Last 7 days' }
    ];

    let period = $state<UsagePeriod>('1week');
    let usage = $state<UsagePoint[]>([]);
    let loading = $state(false);
    let canvas = $state<HTMLCanvasElement | null>(null);

    let total = $derived(usage.reduce((sum, point) => sum + point.count, 0));

    function pad(value: number) {
        return String(value).padStart(2, '0');
    }

    function buildSeries(selected: UsagePeriod) {
        const labels: string[] = [];
        const counts: number[] = [];
        const now = new Date();
        const countFor = (key: string) => usage.find((point) => point.date === key)?.count ?? 0;

        if (selected === '1hr') {
            const base = now.getTime() - (now.getUTCSeconds() * 1000 + now.getUTCMilliseconds());
            for (let offset = 59; offset >= 0; offset--) {
                const date = new Date(base - offset * 60000);
                labels.push(`${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}`);
                counts.push(countFor(`${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}`));
            }
        } else if (selected === '1day') {
            const base = now.getTime() - (now.getUTCMinutes() * 60000 + now.getUTCSeconds() * 1000 + now.getUTCMilliseconds());
            for (let offset = 23; offset >= 0; offset--) {
                const date = new Date(base - offset * 3600000);
                labels.push(`${pad(date.getUTCHours())}:00`);
                counts.push(countFor(`${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}`));
            }
        } else {
            for (let offset = 6; offset >= 0; offset--) {
                const date = new Date();
                date.setUTCDate(date.getUTCDate() - offset);
                const key = date.toISOString().split('T')[0];
                labels.push(key.slice(5));
                counts.push(countFor(key));
            }
        }
        return { labels, counts };
    }

    async function loadUsage(selected: UsagePeriod) {
        loading = true;
        try {
            const res = await fetch(`/api/keys/usage?period=${selected}`);
            usage = res.ok ? await res.json() : [];
        } catch {
            usage = [];
        }
        loading = false;
    }

    $effect(() => {
        loadUsage(period);
    });

    $effect(() => {
        if (!canvas) return;
        const accent = getComputedStyle(document.documentElement).getPropertyValue('--color-accent').trim() || '#3b82f6';
        const muted = getComputedStyle(document.documentElement).getPropertyValue('--color-ink-muted').trim() || '#9ca3af';
        const { labels, counts } = buildSeries(period);
        const chart = new Chart(canvas, {
            type: 'line',
            data: {
                labels,
                datasets: [{
                    label: 'Requests',
                    data: counts,
                    fill: true,
                    backgroundColor: `${accent}22`,
                    borderColor: accent,
                    pointBackgroundColor: accent,
                    pointRadius: period === '1hr' ? 0 : 2,
                    tension: 0.3
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false },
                    tooltip: { callbacks: { label: (item) => ` ${item.raw} request${(item.raw as number) === 1 ? '' : 's'}` } }
                },
                scales: {
                    y: { beginAtZero: true, ticks: { precision: 0, color: muted, font: { size: 10 } }, grid: { color: `${muted}22` } },
                    x: { ticks: { color: muted, font: { size: 10 }, maxTicksLimit: period === '1hr' ? 12 : 8 }, grid: { display: false } }
                }
            }
        });
        return () => chart.destroy();
    });
</script>

<div class="px-5 py-4">
    <div class="mb-3 flex items-center justify-between gap-3">
        <p class="text-sm text-[var(--color-ink-muted)]">
            <span class="font-semibold text-[var(--color-ink)] tabular-nums">{total}</span> requests in the {periods.find((item) => item.value === period)?.title.toLowerCase()}
        </p>
        <div class="flex rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-0.5 text-xs font-semibold" role="radiogroup" aria-label="Usage period">
            {#each periods as item}
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
    </div>
    <div class="h-40">
        {#if loading && usage.length === 0}
            <div class="flex h-full items-center justify-center text-sm text-[var(--color-ink-muted)]">
                <Icon icon="mdi:loading" class="mr-2 animate-spin" /> Loading...
            </div>
        {:else if total === 0}
            <div class="flex h-full items-center justify-center text-sm text-[var(--color-ink-muted)]">
                No requests in this period.
            </div>
        {:else}
            <canvas bind:this={canvas}></canvas>
        {/if}
    </div>
</div>
