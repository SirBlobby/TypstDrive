<script lang="ts">
    import { Chart, LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip, Legend } from 'chart.js';
    import { formatDuration, type TimingPoint } from '#lib/ts/site/performance';

    Chart.register(LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip, Legend);

    let { points }: { points: TimingPoint[] } = $props();

    let canvas = $state<HTMLCanvasElement | null>(null);

    function timeLabel(seconds: number) {
        return new Date(seconds * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }

    function cssColor(name: string, fallback: string) {
        return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback;
    }

    $effect(() => {
        if (!canvas) return;
        const accent = cssColor('--color-accent', '#3b82f6');
        const muted = cssColor('--color-ink-muted', '#9ca3af');
        const slow = '#f59e0b';
        const chart = new Chart(canvas, {
            type: 'line',
            data: {
                labels: points.map((point) => timeLabel(point.time)),
                datasets: [
                    {
                        label: 'Average',
                        data: points.map((point) => point.average_ms),
                        borderColor: accent,
                        backgroundColor: `${accent}22`,
                        fill: true,
                        pointRadius: 0,
                        pointHitRadius: 8,
                        tension: 0.3
                    },
                    {
                        label: 'Slowest 5%',
                        data: points.map((point) => point.p95_ms),
                        borderColor: slow,
                        borderDash: [4, 3],
                        pointRadius: 0,
                        pointHitRadius: 8,
                        tension: 0.3
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                interaction: { mode: 'index', intersect: false },
                plugins: {
                    legend: { position: 'bottom', labels: { color: muted, boxWidth: 12, boxHeight: 2, font: { size: 11 } } },
                    tooltip: {
                        callbacks: {
                            label: (item) => ` ${item.dataset.label}: ${formatDuration(item.raw as number | null)}`,
                            footer: (items) => {
                                const point = points[items[0]?.dataIndex ?? 0];
                                return point ? `${point.count} total, ${point.failures} failed` : '';
                            }
                        }
                    }
                },
                scales: {
                    y: {
                        beginAtZero: true,
                        ticks: { color: muted, font: { size: 10 }, callback: (value) => formatDuration(Number(value)) },
                        grid: { color: `${muted}22` }
                    },
                    x: { ticks: { color: muted, font: { size: 10 }, maxTicksLimit: 8 }, grid: { display: false } }
                }
            }
        });
        return () => chart.destroy();
    });
</script>

<div class="h-44">
    <canvas bind:this={canvas}></canvas>
</div>
