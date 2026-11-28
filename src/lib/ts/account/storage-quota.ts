export type QuotaUnit = 'MB' | 'GB' | 'TB';

export const QUOTA_UNITS: QuotaUnit[] = ['MB', 'GB', 'TB'];

const UNIT_BYTES: Record<QuotaUnit, number> = {
    MB: 1024 ** 2,
    GB: 1024 ** 3,
    TB: 1024 ** 4
};

export type QuotaDraft = { amount: string; unit: QuotaUnit };

export function quotaToDraft(bytes: number): QuotaDraft {
    if (bytes <= 0) return { amount: '0', unit: 'GB' };
    const unit = [...QUOTA_UNITS].reverse().find((candidate) => bytes % UNIT_BYTES[candidate] === 0) ?? 'MB';
    const amount = bytes / UNIT_BYTES[unit];
    return { amount: String(Number.isInteger(amount) ? amount : Number(amount.toFixed(2))), unit };
}

export function draftToQuota(draft: QuotaDraft): number | null {
    const amount = Number(String(draft.amount ?? '').trim());
    if (!Number.isFinite(amount) || amount < 0) return null;
    return Math.round(amount * UNIT_BYTES[draft.unit]);
}

export function usageShare(used: number, quota: number): number {
    if (quota <= 0) return 0;
    return Math.min(100, (used / quota) * 100);
}
