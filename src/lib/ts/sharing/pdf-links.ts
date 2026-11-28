export type PdfLinkKind = 'document' | 'project';

export type PdfLink = {
    id: string;
    slug: string;
    kind: PdfLinkKind;
    item_id: string;
    item_name: string;
    owner_username?: string;
    view_count: number;
    last_viewed_at: string | null;
    created_at: string;
};

export type ItemPdfLink = {
    enabled: boolean;
    can_manage: boolean;
    suggested_slug: string;
    link: PdfLink | null;
};

export type OwnPdfLinks = {
    enabled: boolean;
    max_links_per_user: number;
    links: PdfLink[];
};

export type PdfLinkPolicy = {
    enabled: boolean;
    max_links_per_user: number;
};

export type PdfLinkOverview = {
    policy: PdfLinkPolicy;
    links: PdfLink[];
};

export const SLUG_PATTERN = /^[a-z0-9](?:[a-z0-9-]{1,62}[a-z0-9])$/;

export function pdfLinkUrl(slug: string): string {
    return `${window.location.origin}/pdf/${slug}`;
}

export function itemPageUrl(link: PdfLink): string {
    return `${link.kind === 'project' ? '/project' : '/doc'}/${link.item_id}`;
}

export function normalizeSlug(value: string): string {
    return value.trim().toLowerCase().replace(/\s+/g, '-');
}

async function errorText(res: Response, fallback: string): Promise<string> {
    return (await res.text().catch(() => '')) || fallback;
}

export async function fetchItemPdfLink(kind: PdfLinkKind, itemId: string): Promise<ItemPdfLink | null> {
    const res = await fetch(`/api/pdf-links/${kind}/${itemId}`).catch(() => null);
    return res?.ok ? await res.json() : null;
}

export async function savePdfLink(kind: PdfLinkKind, itemId: string, slug: string): Promise<PdfLink> {
    const res = await fetch(`/api/pdf-links/${kind}/${itemId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ slug })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Could not save the PDF link'));
    return res.json();
}

export async function removePdfLink(kind: PdfLinkKind, itemId: string): Promise<void> {
    const res = await fetch(`/api/pdf-links/${kind}/${itemId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Could not remove the PDF link'));
}

export async function fetchOwnPdfLinks(): Promise<OwnPdfLinks> {
    const res = await fetch('/api/pdf-links');
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load PDF links'));
    return res.json();
}

export async function fetchPdfLinkOverview(): Promise<PdfLinkOverview> {
    const res = await fetch('/api/admin/pdf-links');
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load PDF links'));
    return res.json();
}

export async function savePdfLinkPolicy(policy: PdfLinkPolicy): Promise<PdfLinkPolicy> {
    const res = await fetch('/api/admin/pdf-links/policy', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(policy)
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to save settings'));
    return res.json();
}

export async function adminRemovePdfLink(linkId: string): Promise<void> {
    const res = await fetch(`/api/admin/pdf-links/${linkId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Could not remove the PDF link'));
}

export async function copyText(text: string): Promise<boolean> {
    try {
        await navigator.clipboard.writeText(text);
        return true;
    } catch {
        return false;
    }
}
