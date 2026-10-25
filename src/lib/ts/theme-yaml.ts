import { parse, stringify } from 'yaml';
import { paletteKeys, type ThemeDraft, type ThemeMode, type ThemePalette } from './custom-themes';
import { builtinThemeDraft } from './builtin-palettes';

const MAX_NAME_LENGTH = 40;

export function normalizeColor(value: unknown): string | null {
    if (typeof value !== 'string') return null;
    const color = value.trim().toLowerCase();
    if (/^#[0-9a-f]{6}$/.test(color)) return color;
    if (/^#[0-9a-f]{3}$/.test(color)) return `#${[...color.slice(1)].map((digit) => digit + digit).join('')}`;
    return null;
}

function readPalette(value: unknown, mode: ThemeMode, fallback: ThemePalette): ThemePalette {
    if (value === undefined || value === null) return { ...fallback };
    if (typeof value !== 'object' || Array.isArray(value)) throw new Error(`"${mode}" must be a list of colors.`);
    const entries = value as Record<string, unknown>;
    const palette = { ...fallback };
    for (const key of paletteKeys) {
        if (entries[key] === undefined) continue;
        const color = normalizeColor(entries[key]);
        if (!color) throw new Error(`${mode}.${key} must be a color like "#1e66f5".`);
        palette[key] = color;
    }
    return palette;
}

export function parseThemeYaml(text: string): ThemeDraft {
    let document: unknown;
    try {
        document = parse(text);
    } catch (err) {
        throw new Error(`This file is not valid YAML: ${(err as Error).message}`);
    }
    if (!document || typeof document !== 'object' || Array.isArray(document)) {
        throw new Error('The file needs a name and light and dark colors.');
    }
    const fields = document as Record<string, unknown>;
    if (fields.light == null && fields.dark == null) throw new Error('The file needs light or dark colors.');

    const fallback = builtinThemeDraft('Cerberus');
    return {
        name: typeof fields.name === 'string' ? fields.name.trim().slice(0, MAX_NAME_LENGTH) : '',
        light: readPalette(fields.light, 'light', fallback.light),
        dark: readPalette(fields.dark, 'dark', fallback.dark)
    };
}

export function themeToYaml(draft: ThemeDraft) {
    return stringify({ name: draft.name, light: draft.light, dark: draft.dark });
}

export function downloadThemeYaml(draft: ThemeDraft) {
    const blob = new Blob([themeToYaml(draft)], { type: 'application/yaml' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${(draft.name || 'theme').replace(/[^\w.-]+/g, '-').toLowerCase()}.yml`;
    link.click();
    URL.revokeObjectURL(url);
}
