import { writable } from 'svelte/store';

export interface LatexPolicy {
    enabled: boolean;
}

export const latexEnabledStore = writable(true);

async function request(url: string, method = 'GET', body?: LatexPolicy): Promise<LatexPolicy> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.json();
}

export async function loadLatexPolicy() {
    try {
        const policy = await request('/api/latex/policy');
        latexEnabledStore.set(policy.enabled);
    } catch {
        latexEnabledStore.set(true);
    }
}

export async function fetchLatexAdminPolicy() {
    const policy = await request('/api/admin/latex-policy');
    latexEnabledStore.set(policy.enabled);
    return policy;
}

export async function saveLatexAdminPolicy(policy: LatexPolicy) {
    const saved = await request('/api/admin/latex-policy', 'PUT', policy);
    latexEnabledStore.set(saved.enabled);
    return saved;
}
