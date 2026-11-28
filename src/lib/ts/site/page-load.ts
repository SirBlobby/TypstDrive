let reported = false;

export function reportPageLoad() {
    if (reported || typeof navigator === 'undefined' || typeof performance === 'undefined') return;
    reported = true;
    const body = new Blob([JSON.stringify({ duration_ms: performance.now() })], { type: 'application/json' });
    if (!navigator.sendBeacon?.('/api/metrics/page-load', body)) {
        fetch('/api/metrics/page-load', { method: 'POST', body, headers: { 'Content-Type': 'application/json' }, keepalive: true }).catch(() => {});
    }
}
