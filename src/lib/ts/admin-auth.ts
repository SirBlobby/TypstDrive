export type AuthSettings = {
    password_login_enabled: boolean;
    email_code_login_enabled: boolean;
    require_email_verification: boolean;
    auto_create_accounts: boolean;
    disabled_providers: string[];
    max_failed_logins: number;
    lockout_minutes: number;
    session_days: number;
};

export type EmailStatus = {
    configured: boolean;
    host: string | null;
    port: number | null;
    security: string | null;
    from: string | null;
    has_credentials: boolean;
    problem: string | null;
};

export type ProviderStatus = {
    id: string;
    name: string;
    kind: 'oidc' | 'github';
    enabled: boolean;
    callback_url: string | null;
};

export type AuthenticationOverview = {
    settings: AuthSettings;
    public_url: string | null;
    email: EmailStatus;
    verification_available: boolean;
    providers: ProviderStatus[];
    provider_problems: string[];
};

export type AuditEntry = {
    id: string;
    actor_id: string | null;
    actor_name: string;
    action: string;
    target: string | null;
    details: string | null;
    ip_address: string | null;
    created_at: string;
};

export type AuditPage = {
    entries: AuditEntry[];
    next_before: string | null;
};

async function readJson<T>(res: Response, fallback: string): Promise<T> {
    if (!res.ok) throw new Error((await res.text()) || fallback);
    return res.json();
}

export async function fetchAuthentication(): Promise<AuthenticationOverview> {
    return readJson(await fetch('/api/admin/authentication'), 'Failed to load authentication settings.');
}

export async function saveAuthentication(settings: AuthSettings): Promise<AuthenticationOverview> {
    const res = await fetch('/api/admin/authentication', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings)
    });
    return readJson(res, 'Failed to save settings.');
}

export async function sendTestEmail(recipient: string): Promise<{ sent_to: string }> {
    const res = await fetch('/api/admin/email/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ recipient: recipient || null })
    });
    return readJson(res, 'Sending failed.');
}

export async function fetchAuditLog(before: string | null, action: string): Promise<AuditPage> {
    const params = new URLSearchParams({ limit: '50' });
    if (before) params.set('before', before);
    if (action) params.set('action', action);
    return readJson(await fetch(`/api/admin/audit-log?${params}`), 'Failed to load the audit log.');
}

export function formatTimestamp(value: string): string {
    const date = new Date(value.includes('T') ? value : value.replace(' ', 'T') + 'Z');
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}
