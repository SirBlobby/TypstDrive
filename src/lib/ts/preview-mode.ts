import { derived, writable } from 'svelte/store';

export type PreviewMode = 'split' | 'editor' | 'preview';

export const previewModeOptions: { value: PreviewMode; label: string; icon: string }[] = [
    { value: 'split', label: 'Editor and preview', icon: 'mdi:view-split-vertical' },
    { value: 'editor', label: 'Editor only', icon: 'mdi:file-document-edit-outline' },
    { value: 'preview', label: 'Preview only', icon: 'mdi:eye-outline' }
];

export const previewWindowOption = { label: 'Preview in new window', icon: 'mdi:open-in-new' };

function isPreviewMode(value: string | null): value is PreviewMode {
    return value === 'split' || value === 'editor' || value === 'preview';
}

export const previewModeStore = writable<PreviewMode>('split');
export const previewWindowStore = writable(false);
export const layoutModeStore = derived([previewModeStore, previewWindowStore], ([mode, popped]): PreviewMode => (popped ? 'editor' : mode));
export const previewVisibleStore = derived(layoutModeStore, (mode) => mode !== 'editor');
export const editorVisibleStore = derived(layoutModeStore, (mode) => mode !== 'preview');

export function choosePreviewMode(mode: PreviewMode) {
    previewWindowStore.set(false);
    previewModeStore.set(mode);
}

if (typeof window !== 'undefined') {
    const savedPreviewMode = localStorage.getItem('editor-preview-mode');
    if (isPreviewMode(savedPreviewMode)) previewModeStore.set(savedPreviewMode);
    previewModeStore.subscribe((mode) => localStorage.setItem('editor-preview-mode', mode));
}

export const MIN_SPLIT = 0.2;
export const MAX_SPLIT = 0.8;
export const DEFAULT_SPLIT = 0.5;

export function clampSplit(ratio: number) {
    return Math.min(MAX_SPLIT, Math.max(MIN_SPLIT, ratio));
}

export const splitRatioStore = writable<number>(DEFAULT_SPLIT);

if (typeof window !== 'undefined') {
    const savedSplit = Number.parseFloat(localStorage.getItem('editor-split-ratio') ?? '');
    if (Number.isFinite(savedSplit)) splitRatioStore.set(clampSplit(savedSplit));
    splitRatioStore.subscribe((ratio) => localStorage.setItem('editor-split-ratio', String(ratio)));
}
