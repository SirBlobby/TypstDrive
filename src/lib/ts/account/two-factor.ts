import { postJson, readError } from './auth-options';
import type { TwoFactorMethod } from './sign-in';

export type TwoFactorStatus = {
    method: TwoFactorMethod | null;
    backup_codes_left: number;
    authenticator_available: boolean;
    email_available: boolean;
};

export type AuthenticatorSetup = {
    secret: string;
    uri: string;
    qr_size: number;
    qr_modules: string;
};

export type ActionResult<T> = { ok: true; value: T } | { ok: false; error: string };

export const METHOD_LABELS: Record<TwoFactorMethod, string> = {
    totp: 'Authenticator app',
    email: 'Email codes'
};

export async function fetchTwoFactorStatus(): Promise<TwoFactorStatus | null> {
    const res = await fetch('/api/auth/two-factor').catch(() => null);
    return res?.ok ? res.json() : null;
}

export async function twoFactorAction<T>(path: string, body: unknown, fallback: string): Promise<ActionResult<T>> {
    const res = await postJson(`/api/auth/two-factor${path}`, body).catch(() => null);
    if (!res) return { ok: false, error: 'Network error.' };
    if (!res.ok) return { ok: false, error: await readError(res, fallback) };
    const text = await res.text();
    return { ok: true, value: (text ? JSON.parse(text) : null) as T };
}

export function downloadBackupCodes(codes: string[], siteName: string) {
    const content = `${siteName} backup codes\nEach code works once.\n\n${codes.join('\n')}\n`;
    const link = document.createElement('a');
    link.href = URL.createObjectURL(new Blob([content], { type: 'text/plain' }));
    link.download = 'backup-codes.txt';
    link.click();
    URL.revokeObjectURL(link.href);
}
