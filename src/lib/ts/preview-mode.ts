import { derived, writable } from 'svelte/store';

export type PreviewMode = 'split' | 'editor' | 'preview';

export const previewModeOptions: { value: PreviewMode; label: string; icon: string }[] = [
    { value: 'split', label: 'Editor and preview', icon: 'mdi:view-split-vertical' },
    { value: 'editor', label: 'Editor only', icon: 'mdi:file-document-edit-outline' },
    { value: 'preview', label: 'Preview only', icon: 'mdi:eye-outline' }
];

function isPreviewMode(value: string | null): value is PreviewMode {
    return value === 'split' || value === 'editor' || value === 'preview';
}

export const previewModeStore = writable<PreviewMode>('split');
export const previewVisibleStore = derived(previewModeStore, (mode) => mode !== 'editor');
export const editorVisibleStore = derived(previewModeStore, (mode) => mode !== 'preview');

if (typeof window !== 'undefined') {
    const savedPreviewMode = localStorage.getItem('editor-preview-mode');
    if (isPreviewMode(savedPreviewMode)) previewModeStore.set(savedPreviewMode);
    previewModeStore.subscribe((mode) => localStorage.setItem('editor-preview-mode', mode));
}
