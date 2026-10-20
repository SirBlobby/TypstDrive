export type LinkKind = 'document' | 'project';

export type LinkStatus = {
    name: string;
    public_role: string | null;
    requires_password: boolean;
    expires_at: string | null;
    expired: boolean;
    unlocked: boolean;
};

export type LinkSettings = {
    public_role: string;
    password?: string;
    remove_password?: boolean;
    expires_at?: string;
};

const itemApi: Record<LinkKind, string> = { document: '/api/docs', project: '/api/projects' };

export async function canOpenItem(kind: LinkKind, id: string): Promise<boolean> {
    const res = await fetch(`${itemApi[kind]}/${id}`).catch(() => null);
    return Boolean(res?.ok);
}

export async function fetchLinkStatus(kind: LinkKind, id: string): Promise<LinkStatus | null> {
    const res = await fetch(`/api/share-links/${kind}/${id}`).catch(() => null);
    return res?.ok ? await res.json() : null;
}

export async function unlockLink(kind: LinkKind, id: string, password: string): Promise<string | null> {
    const res = await fetch(`/api/share-links/${kind}/${id}/unlock`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password })
    });
    return res.ok ? null : (await res.text()) || 'Could not unlock the link';
}

export async function saveLinkSettings(kind: LinkKind, id: string, settings: LinkSettings): Promise<LinkStatus> {
    const res = await fetch(`/api/share-links/${kind}/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settings)
    });
    if (!res.ok) throw new Error((await res.text()) || 'Could not save link settings');
    return res.json();
}

export function toLocalInputValue(timestamp: string | null): string {
    if (!timestamp) return '';
    const date = new Date(timestamp.replace(' ', 'T') + 'Z');
    if (Number.isNaN(date.getTime())) return '';
    const offsetMinutes = date.getTimezoneOffset();
    return new Date(date.getTime() - offsetMinutes * 60000).toISOString().slice(0, 16);
}

export function fromLocalInputValue(value: string): string {
    if (!value) return '';
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? '' : date.toISOString();
}
