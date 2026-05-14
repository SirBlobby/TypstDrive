import { writable } from 'svelte/store';
import type { Diagnostic } from './typst-api';

export const themeStore = writable('Catppuccin');
export const darkModeStore = writable(true);
export const connectionStatus = writable('connecting');
export const editorViewStore = writable<any>(null);
export const documentZoomStore = writable(100);
export const commentsSidebarOpen = writable(false);
export const versionHistoryOpen = writable(false);
export const commentReference = writable('');
export const editorErrors = writable<Diagnostic[]>([]);
export const documentStatsStore = writable<{pages: number; words: number; characters: number; characters_excluding_spaces: number} | null>(null);
export const triggerLspReconnect = writable(0);
export const previewOpenStore = writable(true);

export interface AwarenessUser {
    clientId: number;
    name: string;
    color: string;
    colorLight: string;
    isLocal?: boolean;
}

export const connectedUsers = writable<AwarenessUser[]>([]);


if (typeof window !== 'undefined') {
    const savedTheme = localStorage.getItem('editor-theme');
    const savedDark = localStorage.getItem('editor-dark-mode');
    const savedZoom = localStorage.getItem('editor-document-zoom');
    
    if (savedTheme) themeStore.set(savedTheme);
    if (savedDark !== null) darkModeStore.set(savedDark === 'true');
    if (savedZoom !== null) documentZoomStore.set(parseInt(savedZoom, 10));
    
    themeStore.subscribe(value => localStorage.setItem('editor-theme', value));
    darkModeStore.subscribe(value => {
        localStorage.setItem('editor-dark-mode', value.toString());
        if (value) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }
    });
    documentZoomStore.subscribe(value => localStorage.setItem('editor-document-zoom', value.toString()));
}
