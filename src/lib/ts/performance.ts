export type PerformancePeriod = '1h' | '6h' | '24h';

export type TimingKey = 'page_load' | 'api_request' | 'typst_compile' | 'latex_compile';

export interface TimingPoint {
    time: number;
    count: number;
    failures: number;
    average_ms: number | null;
    p95_ms: number | null;
}

export interface TimingSummary {
    count: number;
    failures: number;
    average_ms: number | null;
    median_ms: number | null;
    p95_ms: number | null;
    max_ms: number | null;
    points: TimingPoint[];
}

export interface PerformanceReport {
    uptime_seconds: number;
    instance_id: string;
    system: {
        process_memory_bytes: number | null;
        process_threads: number | null;
        process_cpu_percent: number | null;
        cpu_cores: number;
        load_average: [number, number, number] | null;
        memory_total_bytes: number | null;
        memory_available_bytes: number | null;
        disk_path: string;
        disk_total_bytes: number | null;
        disk_free_bytes: number | null;
    };
    activity: {
        live_documents: number;
        live_connections: number;
        latex_compiles_running: number;
        latex_compiles_allowed: number;
    };
    content: {
        users: number;
        documents: number;
        projects: number;
        files: number;
        file_bytes: number;
    };
    timings: Record<TimingKey, TimingSummary>;
}

export const TIMING_DETAILS: { key: TimingKey; title: string; description: string; icon: string }[] = [
    {
        key: 'page_load',
        title: 'Page load',
        description: 'Time from opening TypstDrive in the browser until the app is ready, as reported by signed-in visitors.',
        icon: 'mdi:web-clock'
    },
    {
        key: 'api_request',
        title: 'Server responses',
        description: 'Time the server takes to answer requests from the app and the render API. Failures are server errors.',
        icon: 'mdi:swap-horizontal'
    },
    {
        key: 'typst_compile',
        title: 'Typst compiles',
        description: 'Time spent compiling Typst documents for previews and exports. Failures are documents with errors.',
        icon: 'mdi:file-document-edit-outline'
    },
    {
        key: 'latex_compile',
        title: 'LaTeX compiles',
        description: 'Time Tectonic takes to compile LaTeX documents. Failures are documents with errors or compiles that were stopped.',
        icon: 'mdi:sigma'
    }
];

export async function fetchPerformance(period: PerformancePeriod): Promise<PerformanceReport> {
    const res = await fetch(`/api/admin/performance?period=${period}`);
    if (!res.ok) throw new Error((await res.text()) || 'Failed to load performance details.');
    return res.json();
}

export function formatDuration(milliseconds: number | null) {
    if (milliseconds === null) return '-';
    if (milliseconds < 1) return '<1 ms';
    if (milliseconds < 1000) return `${Math.round(milliseconds)} ms`;
    if (milliseconds < 60000) return `${(milliseconds / 1000).toFixed(milliseconds < 10000 ? 2 : 1)} s`;
    return `${(milliseconds / 60000).toFixed(1)} min`;
}

export function formatUptime(seconds: number) {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
}

export function failureShare(summary: TimingSummary) {
    return summary.count > 0 ? (summary.failures / summary.count) * 100 : 0;
}
