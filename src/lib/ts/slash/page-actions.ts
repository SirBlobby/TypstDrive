import { get, writable } from 'svelte/store';
import { commentsSidebarOpen } from '../store';
import { choosePreviewMode, layoutModeStore, previewModeOptions, previewWindowOption, previewWindowStore } from '../preview-mode';
import { suggestingModeStore } from '../suggestions';
import { exportFormatsFor, type ExportFormat } from '../typst-api';
import type { SlashCommand } from './slash-command-types';

type ConvertedFormat = 'docx' | 'latex' | 'markdown';

export interface PageActionHandlers {
    sourceLabel: string;
    canEdit: boolean;
    isLatex: boolean;
    exportAs: (format: ExportFormat | 'source') => void;
    exportConverted: (format: ConvertedFormat) => void;
    print: () => void;
    openShare?: () => void;
    openPresentation?: () => void;
    openSpeakerView?: () => void;
    openPageSettings?: () => void;
    openCitations?: () => void;
    openInfo?: () => void;
    openHistory?: () => void;
    openGit?: () => void;
    saveVersion?: () => void;
    makeCopy?: () => void;
    rename?: () => void;
    downloadArchive?: () => void;
}

export const pageActionsStore = writable<SlashCommand[]>([]);

export function registerPageActions(actions: SlashCommand[]) {
    pageActionsStore.set(actions);
    return () => pageActionsStore.update((current) => (current === actions ? [] : current));
}

const convertedFormats: { format: ConvertedFormat; label: string; icon: string; keywords: string[] }[] = [
    { format: 'docx', label: 'Export as Word', icon: 'mdi:file-word-box', keywords: ['docx', 'microsoft'] },
    { format: 'latex', label: 'Export as LaTeX', icon: 'mdi:sigma', keywords: ['tex'] },
    { format: 'markdown', label: 'Export as Markdown', icon: 'mdi:language-markdown', keywords: ['md'] }
];

function exportCommands(handlers: PageActionHandlers): SlashCommand[] {
    const rendered: SlashCommand[] = exportFormatsFor(handlers.isLatex).map((option) => ({
        id: `export-${option.format}`,
        label: `Export as ${option.format.toUpperCase()}`,
        description: `Download a ${option.label}`,
        icon: option.icon,
        group: 'File',
        keywords: ['download', 'save', option.format],
        run: () => handlers.exportAs(option.format)
    }));
    const converted: SlashCommand[] = convertedFormats.map((option) => ({
        id: `export-${option.format}`,
        label: option.label,
        description: 'Convert and download',
        icon: option.icon,
        group: 'File',
        keywords: ['download', 'convert', ...option.keywords],
        run: () => handlers.exportConverted(option.format)
    }));
    return [
        ...rendered,
        {
            id: 'export-source',
            label: 'Download source',
            description: handlers.sourceLabel,
            icon: 'mdi:code-braces',
            group: 'File',
            keywords: ['download', 'source', 'code'],
            run: () => handlers.exportAs('source')
        },
        ...converted,
        { id: 'print', label: 'Print', description: 'Print the document', icon: 'mdi:printer-outline', group: 'File', keywords: ['paper'], run: handlers.print }
    ];
}

function modeCommands(handlers: PageActionHandlers): SlashCommand[] {
    if (!handlers.canEdit) return [];
    return [
        {
            id: 'mode-editing',
            label: 'Switch to editing',
            description: 'Change the text directly',
            icon: 'mdi:pencil-outline',
            group: 'Review',
            keywords: ['mode', 'edit'],
            isAvailable: () => get(suggestingModeStore),
            run: () => suggestingModeStore.set(false)
        },
        {
            id: 'mode-suggesting',
            label: 'Switch to suggesting',
            description: 'Edits become suggestions',
            icon: 'mdi:file-document-edit-outline',
            group: 'Review',
            keywords: ['mode', 'suggest', 'track changes'],
            isAvailable: () => !get(suggestingModeStore),
            run: () => suggestingModeStore.set(true)
        }
    ];
}

function viewCommands(): SlashCommand[] {
    const layouts: SlashCommand[] = previewModeOptions.map((option) => ({
        id: `view-${option.value}`,
        label: option.label,
        description: 'Change the layout',
        icon: option.icon,
        group: 'View',
        keywords: ['view', 'layout', 'preview', option.value],
        isAvailable: () => get(previewWindowStore) || get(layoutModeStore) !== option.value,
        run: () => choosePreviewMode(option.value)
    }));
    return [
        ...layouts,
        {
            id: 'view-window',
            label: previewWindowOption.label,
            description: 'Move the preview into its own window',
            icon: previewWindowOption.icon,
            group: 'View',
            keywords: ['view', 'layout', 'preview', 'window', 'pop out', 'monitor'],
            isAvailable: () => !get(previewWindowStore),
            run: () => previewWindowStore.set(true)
        },
        {
            id: 'comments-panel',
            label: 'Comments panel',
            description: 'Show or hide comments and suggestions',
            icon: 'mdi:comment-text-multiple-outline',
            group: 'View',
            keywords: ['sidebar', 'review', 'suggestions'],
            run: () => commentsSidebarOpen.update((open) => !open)
        }
    ];
}

function optionalCommand(run: (() => void) | undefined, command: Omit<SlashCommand, 'run'>): SlashCommand[] {
    return run ? [{ ...command, run }] : [];
}

export function buildPageActions(handlers: PageActionHandlers): SlashCommand[] {
    return [
        ...modeCommands(handlers),
        ...viewCommands(),
        ...optionalCommand(handlers.openPresentation, { id: 'present', label: 'Present', description: 'Show the pages as slides', icon: 'mdi:presentation-play', group: 'View', keywords: ['slides', 'presentation', 'fullscreen'] }),
        ...optionalCommand(handlers.openSpeakerView, { id: 'speaker', label: 'Speaker view', description: 'Present in a new window', icon: 'mdi:account-voice', group: 'View', keywords: ['slides', 'presentation', 'notes', 'presenter'] }),
        ...exportCommands(handlers),
        ...optionalCommand(handlers.saveVersion, { id: 'save-version', label: 'Save version', description: 'Keep a snapshot in the history', icon: 'mdi:content-save-outline', group: 'File', keywords: ['snapshot', 'checkpoint'], shortcut: 'saveVersion' }),
        ...optionalCommand(handlers.openHistory, { id: 'version-history', label: 'Version history', description: 'Look back at older versions', icon: 'mdi:history', group: 'File', keywords: ['versions', 'restore', 'revisions'] }),
        ...optionalCommand(handlers.downloadArchive, { id: 'download-zip', label: 'Download project', description: 'All files as a .zip', icon: 'mdi:folder-zip-outline', group: 'File', keywords: ['zip', 'archive', 'download'] }),
        ...optionalCommand(handlers.makeCopy, { id: 'make-copy', label: 'Make a copy', description: 'Duplicate into your drive', icon: 'mdi:content-copy', group: 'File', keywords: ['duplicate', 'clone'] }),
        ...optionalCommand(handlers.rename, { id: 'rename', label: 'Rename', description: 'Change the name', icon: 'mdi:rename-outline', group: 'File', keywords: ['title', 'name'] }),
        ...optionalCommand(handlers.openShare, { id: 'share', label: 'Share', description: 'Choose who can open it', icon: 'mdi:share-variant-outline', group: 'File', keywords: ['invite', 'link', 'collaborate'] }),
        ...optionalCommand(handlers.openInfo, { id: 'info', label: 'Info', description: 'Size, owner and dates', icon: 'mdi:information-outline', group: 'File', keywords: ['details', 'properties'] }),
        ...optionalCommand(handlers.openPageSettings, { id: 'page-settings', label: 'Page settings', description: 'Paper size, margins and font', icon: 'mdi:file-cog-outline', group: 'Tools', keywords: ['margins', 'paper', 'size', 'orientation'] }),
        ...optionalCommand(handlers.openCitations, { id: 'citations', label: 'Citations', description: 'Add and manage references', icon: 'ph:quotes', group: 'Tools', keywords: ['bibliography', 'zotero', 'doi', 'references'] }),
        ...optionalCommand(handlers.openGit, { id: 'git', label: 'Git sync', description: 'Pull and push to a repository', icon: 'mdi:git', group: 'Tools', keywords: ['github', 'gitlab', 'repository', 'push', 'pull'] })
    ];
}
