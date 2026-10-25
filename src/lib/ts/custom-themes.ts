import { writable } from 'svelte/store';
import type { ThemeColors } from './themes';

export type ThemeScope = 'site' | 'personal';
export type ThemeMode = 'light' | 'dark';

export const appColorFields = [
    { key: 'surface', label: 'Surface' },
    { key: 'surface_muted', label: 'Background' },
    { key: 'surface_sunken', label: 'Sunken' },
    { key: 'line', label: 'Borders' },
    { key: 'ink', label: 'Text' },
    { key: 'ink_muted', label: 'Muted text' },
    { key: 'accent', label: 'Accent' },
    { key: 'accent_soft', label: 'Soft accent' },
    { key: 'danger', label: 'Danger' },
    { key: 'success', label: 'Success' }
] as const;

export const editorColorFields = [
    { key: 'editor_background', label: 'Background' },
    { key: 'editor_text', label: 'Text' },
    { key: 'selection', label: 'Selection' },
    { key: 'active_line', label: 'Active line' },
    { key: 'cursor', label: 'Cursor' },
    { key: 'keyword', label: 'Keywords' },
    { key: 'string', label: 'Strings' },
    { key: 'number', label: 'Numbers' },
    { key: 'comment', label: 'Comments' },
    { key: 'variable', label: 'Variables' },
    { key: 'function', label: 'Functions' }
] as const;

export type PaletteKey = (typeof appColorFields)[number]['key'] | (typeof editorColorFields)[number]['key'];
export type ThemePalette = Record<PaletteKey, string>;

export const paletteKeys: PaletteKey[] = [...appColorFields, ...editorColorFields].map((field) => field.key);

export interface ThemeDraft {
    name: string;
    light: ThemePalette;
    dark: ThemePalette;
}

export interface CustomTheme extends ThemeDraft {
    id: string;
}

export interface ThemeLibrary {
    site: CustomTheme[];
    personal: CustomTheme[];
    personal_enabled: boolean;
}

const LIBRARY_STORAGE_KEY = 'custom-themes';
const emptyLibrary: ThemeLibrary = { site: [], personal: [], personal_enabled: false };

function storedLibrary(): ThemeLibrary {
    if (typeof window === 'undefined') return emptyLibrary;
    try {
        const saved = localStorage.getItem(LIBRARY_STORAGE_KEY);
        return saved ? { ...emptyLibrary, ...JSON.parse(saved) } : emptyLibrary;
    } catch {
        return emptyLibrary;
    }
}

export const customThemesStore = writable<ThemeLibrary>(storedLibrary());

if (typeof window !== 'undefined') {
    customThemesStore.subscribe((library) => {
        try {
            localStorage.setItem(LIBRARY_STORAGE_KEY, JSON.stringify(library));
        } catch {
            return;
        }
    });
}

export function themeKey(scope: ThemeScope, id: string) {
    return `${scope}:${id}`;
}

export function isCustomThemeKey(key: string) {
    return key.startsWith('site:') || key.startsWith('personal:');
}

export function findCustomTheme(library: ThemeLibrary, key: string): CustomTheme | null {
    const separator = key.indexOf(':');
    const scope = key.slice(0, separator);
    const id = key.slice(separator + 1);
    const themes = scope === 'site' ? library.site : scope === 'personal' ? library.personal : [];
    return themes.find((theme) => theme.id === id) ?? null;
}

export function editorColorsFrom(palette: ThemePalette): ThemeColors {
    return {
        background: palette.editor_background,
        text: palette.editor_text,
        selection: palette.selection,
        activeLine: palette.active_line,
        cursor: palette.cursor,
        keyword: palette.keyword,
        string: palette.string,
        number: palette.number,
        comment: palette.comment,
        variable: palette.variable,
        function: palette.function
    };
}

export function cloneDraft(draft: ThemeDraft): ThemeDraft {
    return { name: draft.name, light: { ...draft.light }, dark: { ...draft.dark } };
}

async function request<T>(url: string, method = 'GET', body?: unknown): Promise<T> {
    const res = await fetch(url, {
        method,
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error((await res.text().catch(() => '')) || 'Request failed');
    return res.status === 204 ? (undefined as T) : res.json();
}

function themesUrl(scope: ThemeScope, id?: string) {
    const base = scope === 'site' ? '/api/admin/themes' : '/api/themes';
    return id ? `${base}/${encodeURIComponent(id)}` : base;
}

export async function loadCustomThemes() {
    try {
        customThemesStore.set(await request<ThemeLibrary>('/api/themes'));
        return true;
    } catch {
        return false;
    }
}

export async function saveTheme(scope: ThemeScope, draft: ThemeDraft, id?: string) {
    const saved = await request<CustomTheme>(themesUrl(scope, id), id ? 'PUT' : 'POST', draft);
    await loadCustomThemes();
    return saved;
}

export async function deleteTheme(scope: ThemeScope, id: string) {
    await request<void>(themesUrl(scope, id), 'DELETE');
    await loadCustomThemes();
}

export type ThemePolicy = { personal_enabled: boolean };

export const fetchThemePolicy = () => request<ThemePolicy>('/api/admin/theme-policy');

export async function saveThemePolicy(policy: ThemePolicy) {
    const saved = await request<ThemePolicy>('/api/admin/theme-policy', 'PUT', policy);
    await loadCustomThemes();
    return saved;
}
