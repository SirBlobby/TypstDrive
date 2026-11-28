import { writable } from 'svelte/store';

export const PROJECT_URL = 'https://github.com/SirBlobby/TypstDrive';
const DEFAULT_LOGO_URL = '/favicon.png';

export type BrandingSettings = {
    site_name: string;
    tagline: string;
    accent_color: string;
    login_message: string;
};

export type Branding = BrandingSettings & {
    logo_updated_at: string | null;
    customized: boolean;
};

export const defaultBranding: Branding = {
    site_name: 'TypstDrive',
    tagline: 'A collaborative Typst editor and document manager.',
    accent_color: '',
    login_message: '',
    logo_updated_at: null,
    customized: false
};

export const brandingStore = writable<Branding>(defaultBranding);

export function toSettings(branding: Branding): BrandingSettings {
    return {
        site_name: branding.site_name,
        tagline: branding.tagline,
        accent_color: branding.accent_color,
        login_message: branding.login_message
    };
}

export function logoUrl(branding: Branding) {
    return branding.logo_updated_at
        ? `/api/branding/logo?v=${encodeURIComponent(branding.logo_updated_at)}`
        : DEFAULT_LOGO_URL;
}

export async function loadBranding() {
    try {
        const res = await fetch('/api/branding');
        if (res.ok) brandingStore.set(await res.json());
    } catch {
        brandingStore.set(defaultBranding);
    }
}

export function applyAccentColor(color: string) {
    const root = document.documentElement;
    if (color) {
        root.style.setProperty('--color-accent', color);
        root.style.setProperty('--color-accent-soft', `color-mix(in srgb, ${color} 18%, var(--color-surface))`);
    } else {
        root.style.removeProperty('--color-accent');
        root.style.removeProperty('--color-accent-soft');
    }
}
