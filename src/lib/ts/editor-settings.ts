import { writable } from 'svelte/store';
import { isMinimapSize, type MinimapSize } from './minimap/minimap-sizes';

export interface EditorSettings {
    fontSize: number;
    fontFamily: string;
    spellCheck: boolean;
    spellCheckLanguage: string;
    lineNumbers: boolean;
    minimap: boolean;
    minimapSize: MinimapSize;
}

export const MIN_FONT_SIZE = 10;
export const MAX_FONT_SIZE = 28;

export const spellCheckLanguages = [
    { code: 'en-US', label: 'English (US)' },
    { code: 'en-GB', label: 'English (UK)' },
    { code: 'es', label: 'Spanish' },
    { code: 'fr', label: 'French' },
    { code: 'de', label: 'German' },
    { code: 'it', label: 'Italian' },
    { code: 'pt', label: 'Portuguese' },
    { code: 'nl', label: 'Dutch' }
];

export const defaultEditorSettings: EditorSettings = {
    fontSize: 14,
    fontFamily: 'System monospace',
    spellCheck: false,
    spellCheckLanguage: 'en-US',
    lineNumbers: true,
    minimap: false,
    minimapSize: 'medium'
};

const STORAGE_KEY = 'editor-settings';

function clampFontSize(size: number) {
    if (!Number.isFinite(size)) return defaultEditorSettings.fontSize;
    return Math.min(MAX_FONT_SIZE, Math.max(MIN_FONT_SIZE, Math.round(size)));
}

function loadSaved(): EditorSettings {
    if (typeof window === 'undefined') return defaultEditorSettings;
    try {
        const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
        const merged = { ...defaultEditorSettings, ...saved };
        return {
            ...merged,
            fontSize: clampFontSize(Number(merged.fontSize)),
            minimapSize: isMinimapSize(merged.minimapSize) ? merged.minimapSize : defaultEditorSettings.minimapSize
        };
    } catch {
        return defaultEditorSettings;
    }
}

export const editorSettingsStore = writable<EditorSettings>(loadSaved());

if (typeof window !== 'undefined') {
    editorSettingsStore.subscribe((settings) => {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
        } catch {}
    });
}

export function updateEditorSettings(changes: Partial<EditorSettings>) {
    editorSettingsStore.update((settings) => {
        const next = { ...settings, ...changes };
        return { ...next, fontSize: clampFontSize(next.fontSize) };
    });
}
