export const inputClass =
    'w-full sm:max-w-xs bg-[var(--color-surface)] border border-[var(--color-line)] text-sm text-[var(--color-ink)] rounded-md px-3 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors disabled:opacity-60';

export const primaryButtonClass =
    'inline-flex items-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-white transition hover:opacity-90 disabled:opacity-50';

export const secondaryButtonClass =
    'inline-flex items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2 text-sm font-medium text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-sunken)] disabled:opacity-50';

export const dangerButtonClass =
    'inline-flex items-center gap-2 rounded-md bg-[var(--color-danger)] px-3 py-1.5 text-xs font-semibold text-white transition hover:opacity-90 disabled:opacity-50';

export function formatBytes(bytes: number) {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
    return `${parseFloat((bytes / Math.pow(1024, index)).toFixed(index === 0 ? 0 : 1))} ${units[index]}`;
}

export function formatDate(value: string) {
    const normalized = value.includes('T') || value.endsWith('Z') ? value : value.replace(' ', 'T') + 'Z';
    const date = new Date(normalized);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}
