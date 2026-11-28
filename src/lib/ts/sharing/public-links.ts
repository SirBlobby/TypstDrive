export type PublicLinkKind = 'document' | 'project';

export type LinkFormat = 'pdf' | 'slides' | 'speaker';

export type FormatStatus = {
    enabled: boolean;
    views: number;
    last_viewed_at: string | null;
};

export type PublicLink = {
    id: string;
    slug: string;
    kind: PublicLinkKind;
    item_id: string;
    item_name: string;
    owner_username?: string;
    created_at: string;
    pdf: FormatStatus;
    slides: FormatStatus;
    speaker: FormatStatus;
    has_speaker_password: boolean;
};

export type PublicLinkPolicy = {
    pdf_enabled: boolean;
    slides_enabled: boolean;
    speaker_enabled: boolean;
    max_links_per_user: number;
};

export type ItemPublicLink = {
    policy: PublicLinkPolicy;
    can_manage: boolean;
    suggested_slug: string;
    link: PublicLink | null;
};

export type OwnPublicLinks = {
    policy: PublicLinkPolicy;
    links: PublicLink[];
};

export type PublicLinkOverview = {
    policy: PublicLinkPolicy;
    links: PublicLink[];
};

export type LinkSettings = {
    slug: string;
    pdf_enabled: boolean;
    slides_enabled: boolean;
    speaker_enabled: boolean;
    speaker_password?: string;
};

export const FORMATS: { format: LinkFormat; label: string; icon: string; description: string }[] = [
    { format: 'pdf', label: 'PDF', icon: 'mdi:file-pdf-box', description: 'Anyone with the link can download the latest PDF.' },
    { format: 'slides', label: 'Slides', icon: 'mdi:presentation-play', description: 'Anyone with the link can present the slides. Speaker notes are never included.' },
    { format: 'speaker', label: 'Speaker view', icon: 'mdi:account-voice', description: 'Slides with speaker notes, timer and audience window, protected by a password.' }
];

export const SLUG_PATTERN = /^[a-z0-9](?:[a-z0-9-]{1,62}[a-z0-9])$/;

export function formatPath(format: LinkFormat, slug: string): string {
    if (format === 'pdf') return `/pdf/${slug}`;
    return format === 'slides' ? `/slides/${slug}` : `/slides/${slug}/speaker`;
}

export function formatUrl(format: LinkFormat, slug: string): string {
    return `${window.location.origin}${formatPath(format, slug)}`;
}

export function policyAllows(policy: PublicLinkPolicy, format: LinkFormat): boolean {
    return policy[`${format}_enabled`];
}

export function anyFormatAllowed(policy: PublicLinkPolicy): boolean {
    return policy.pdf_enabled || policy.slides_enabled || policy.speaker_enabled;
}

export function settingsOf(link: PublicLink): LinkSettings {
    return {
        slug: link.slug,
        pdf_enabled: link.pdf.enabled,
        slides_enabled: link.slides.enabled,
        speaker_enabled: link.speaker.enabled
    };
}

export function totalViews(link: PublicLink): number {
    return link.pdf.views + link.slides.views + link.speaker.views;
}

export function itemPageUrl(link: PublicLink): string {
    return `${link.kind === 'project' ? '/project' : '/doc'}/${link.item_id}`;
}

export function normalizeSlug(value: string): string {
    return value.trim().toLowerCase().replace(/\s+/g, '-');
}

async function errorText(res: Response, fallback: string): Promise<string> {
    return (await res.text().catch(() => '')) || fallback;
}

async function sendJson(url: string, method: string, body: unknown): Promise<Response> {
    return fetch(url, { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
}

export async function fetchItemPublicLink(kind: PublicLinkKind, itemId: string): Promise<ItemPublicLink | null> {
    const res = await fetch(`/api/public-links/${kind}/${itemId}`).catch(() => null);
    return res?.ok ? await res.json() : null;
}

export async function savePublicLink(kind: PublicLinkKind, itemId: string, settings: LinkSettings): Promise<PublicLink> {
    const res = await sendJson(`/api/public-links/${kind}/${itemId}`, 'PUT', settings);
    if (!res.ok) throw new Error(await errorText(res, 'Could not save the public link'));
    return res.json();
}

export async function removePublicLink(kind: PublicLinkKind, itemId: string): Promise<void> {
    const res = await fetch(`/api/public-links/${kind}/${itemId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Could not remove the public link'));
}

export async function fetchOwnPublicLinks(): Promise<OwnPublicLinks> {
    const res = await fetch('/api/public-links');
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load public links'));
    return res.json();
}

export async function fetchPublicLinkOverview(): Promise<PublicLinkOverview> {
    const res = await fetch('/api/admin/public-links');
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load public links'));
    return res.json();
}

export async function savePublicLinkPolicy(policy: PublicLinkPolicy): Promise<PublicLinkPolicy> {
    const res = await sendJson('/api/admin/public-links/policy', 'PUT', policy);
    if (!res.ok) throw new Error(await errorText(res, 'Failed to save settings'));
    return res.json();
}

export async function adminRemovePublicLink(linkId: string): Promise<void> {
    const res = await fetch(`/api/admin/public-links/${linkId}`, { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Could not remove the public link'));
}

export async function copyText(text: string): Promise<boolean> {
    try {
        await navigator.clipboard.writeText(text);
        return true;
    } catch {
        return false;
    }
}
