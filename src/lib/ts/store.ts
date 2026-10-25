import { writable } from 'svelte/store';
import type { Diagnostic } from './typst-api';

export const themeStore = writable('Cerberus');
export const darkModeStore = writable(true);
export type TextSize = 'small' | 'default' | 'large';
export const textSizeStore = writable<TextSize>('default');
export const highContrastStore = writable(false);
export const driveSidebarCollapsedStore = writable(false);
export const settingsSidebarCollapsedStore = writable(false);
export const fileTreeCollapsedStore = writable(false);
export const driveViewModeStore = writable<'grid' | 'list'>('grid');
export const connectionStatus = writable('connecting');
export const editorViewStore = writable<any>(null);
export const documentZoomStore = writable(100);
export const commentsSidebarOpen = writable(false);
export const editorErrors = writable<Diagnostic[]>([]);
export const documentStatsStore = writable<{pages: number; words: number; characters: number; characters_excluding_spaces: number} | null>(null);
export const triggerLspReconnect = writable(0);

export interface AwarenessUser {
    clientId: number;
    id?: string | null;
    name: string;
    avatar?: string | null;
    color: string;
    colorLight: string;
    session?: string;
    activeFile?: string | null;
    isLocal?: boolean;
}

export const connectedUsers = writable<AwarenessUser[]>([]);


if (typeof window !== 'undefined') {
    const savedTheme = localStorage.getItem('editor-theme');
    const savedDark = localStorage.getItem('editor-dark-mode');
    const savedZoom = localStorage.getItem('editor-document-zoom');
    const savedTextSize = localStorage.getItem('text-size');
    const savedHighContrast = localStorage.getItem('high-contrast');
    const savedDriveSidebarCollapsed = localStorage.getItem('drive-sidebar-collapsed');
    const savedSettingsSidebarCollapsed = localStorage.getItem('settings-sidebar-collapsed');
    const savedFileTreeCollapsed = localStorage.getItem('file-tree-collapsed');
    const savedDriveViewMode = localStorage.getItem('drive-view-mode');

    if (savedTheme) themeStore.set(savedTheme);
    if (savedDark !== null) darkModeStore.set(savedDark === 'true');
    if (savedZoom !== null) documentZoomStore.set(parseInt(savedZoom, 10));
    if (savedTextSize === 'small' || savedTextSize === 'large') textSizeStore.set(savedTextSize);
    if (savedHighContrast !== null) highContrastStore.set(savedHighContrast === 'true');
    if (savedDriveSidebarCollapsed !== null) driveSidebarCollapsedStore.set(savedDriveSidebarCollapsed === 'true');
    if (savedSettingsSidebarCollapsed !== null) settingsSidebarCollapsedStore.set(savedSettingsSidebarCollapsed === 'true');
    if (savedFileTreeCollapsed !== null) fileTreeCollapsedStore.set(savedFileTreeCollapsed === 'true');
    if (savedDriveViewMode === 'grid' || savedDriveViewMode === 'list') driveViewModeStore.set(savedDriveViewMode);

    themeStore.subscribe(value => {
        localStorage.setItem('editor-theme', value);
        document.documentElement.dataset.colorTheme = value;
    });
    darkModeStore.subscribe(value => {
        localStorage.setItem('editor-dark-mode', value.toString());
        document.documentElement.dataset.theme = value ? 'dark' : 'light';
        document.documentElement.classList.toggle('dark', value);
    });
    documentZoomStore.subscribe(value => localStorage.setItem('editor-document-zoom', value.toString()));
    driveSidebarCollapsedStore.subscribe(value => localStorage.setItem('drive-sidebar-collapsed', value.toString()));
    settingsSidebarCollapsedStore.subscribe(value => localStorage.setItem('settings-sidebar-collapsed', value.toString()));
    fileTreeCollapsedStore.subscribe(value => localStorage.setItem('file-tree-collapsed', value.toString()));
    driveViewModeStore.subscribe(value => localStorage.setItem('drive-view-mode', value));
    textSizeStore.subscribe(value => {
        localStorage.setItem('text-size', value);
        document.documentElement.dataset.textSize = value;
    });
    highContrastStore.subscribe(value => {
        localStorage.setItem('high-contrast', value.toString());
        if (value) {
            document.documentElement.dataset.contrast = 'high';
        } else {
            delete document.documentElement.dataset.contrast;
        }
    });
}
