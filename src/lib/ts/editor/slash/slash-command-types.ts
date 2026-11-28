import type { EditorView } from '@codemirror/view';
import type { KeybindActionId } from '../keybinds/keybind-actions';

export type SlashGroup = 'Text' | 'Lists' | 'Insert' | 'Math' | 'Layout' | 'Review' | 'View' | 'File' | 'Tools';

export const slashGroupOrder: SlashGroup[] = ['Text', 'Lists', 'Insert', 'Math', 'Layout', 'Review', 'View', 'File', 'Tools'];

export interface SlashContext {
    insertDialogs: boolean;
    embedDocuments: boolean;
    canComment: boolean;
    commentMarker: string | null;
}

export interface SlashCommand {
    id: string;
    label: string;
    description: string;
    icon: string;
    group: SlashGroup;
    keywords: string[];
    shortcut?: KeybindActionId;
    isAvailable?: (context: SlashContext) => boolean;
    run: (view: EditorView, context: SlashContext) => void;
}
