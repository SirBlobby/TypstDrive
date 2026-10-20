export type ApiKey = {
    id: string;
    name: string;
    key_prefix: string;
    created_at: string;
    last_used_at: string | null;
    rate_limit: number;
    scopes: string;
    expires_at: string | null;
    custom_rate_limit: number | null;
};

export type CreatedApiKey = {
    id: string;
    name: string;
    key: string;
    prefix: string;
    created_at: string;
    scopes: string;
    expires_at: string | null;
    rate_limit: number;
    custom_rate_limit: number | null;
};

export const API_SCOPES = [
    { id: 'render', label: 'Render', description: 'Compile Typst code with POST /v1/render.' },
    { id: 'read', label: 'Read only', description: 'List and read your documents and projects.' }
];

export const EXPIRY_CHOICES = [
    { days: 0, label: 'Never' },
    { days: 7, label: 'In 7 days' },
    { days: 30, label: 'In 30 days' },
    { days: 90, label: 'In 90 days' },
    { days: 365, label: 'In 1 year' }
];

export function keyFromCreated(created: CreatedApiKey): ApiKey {
    return {
        id: created.id,
        name: created.name,
        key_prefix: created.prefix,
        created_at: created.created_at,
        last_used_at: null,
        rate_limit: created.rate_limit,
        scopes: created.scopes,
        expires_at: created.expires_at,
        custom_rate_limit: created.custom_rate_limit
    };
}

export function scopeLabels(scopes: string): string[] {
    return scopes
        .split(',')
        .map((scope) => API_SCOPES.find((known) => known.id === scope.trim())?.label ?? scope.trim())
        .filter(Boolean);
}

export function isExpired(expiresAt: string | null): boolean {
    if (!expiresAt) return false;
    return new Date(expiresAt.replace(' ', 'T') + 'Z').getTime() <= Date.now();
}
