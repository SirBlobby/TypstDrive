export type SignInProvider = {
    id: string;
    name: string;
    kind: 'oidc' | 'github';
};

export type AuthOptions = {
    password_login_enabled: boolean;
    registration_enabled: boolean;
    password_reset_available: boolean;
    email_code_login_available: boolean;
    email_verification_available: boolean;
    verification_required: boolean;
    providers: SignInProvider[];
};

const fallbackOptions: AuthOptions = {
    password_login_enabled: true,
    registration_enabled: true,
    password_reset_available: false,
    email_code_login_available: false,
    email_verification_available: false,
    verification_required: false,
    providers: []
};

export async function fetchAuthOptions(): Promise<AuthOptions> {
    try {
        const res = await fetch('/api/auth/options');
        if (res.ok) return await res.json();
    } catch {}
    return fallbackOptions;
}

export function providerIcon(provider: SignInProvider): string {
    if (provider.kind === 'github') return 'mdi:github';
    if (provider.id === 'google') return 'mdi:google';
    return 'mdi:shield-account-outline';
}

export function signInUrl(providerId: string, options: { redirect?: string; link?: boolean } = {}): string {
    const params = new URLSearchParams();
    if (options.redirect) params.set('redirect', options.redirect);
    if (options.link) params.set('mode', 'link');
    const query = params.toString();
    return `/api/auth/sso/${encodeURIComponent(providerId)}/start${query ? `?${query}` : ''}`;
}

export async function readError(res: Response, fallback: string): Promise<string> {
    const text = await res.text().catch(() => '');
    return text || fallback;
}

export async function postJson(url: string, body: unknown, method = 'POST'): Promise<Response> {
    return fetch(url, {
        method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body)
    });
}
