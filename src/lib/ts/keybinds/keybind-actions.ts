export type KeybindActionId =
    | 'bold'
    | 'italic'
    | 'code'
    | 'inlineMath'
    | 'toggleComment'
    | 'find'
    | 'replace'
    | 'addComment'
    | 'saveVersion';

export interface KeybindAction {
    id: KeybindActionId;
    label: string;
    defaultKey: string;
}

export const keybindActions: KeybindAction[] = [
    { id: 'bold', label: 'Bold', defaultKey: 'Mod-b' },
    { id: 'italic', label: 'Italic', defaultKey: 'Mod-i' },
    { id: 'code', label: 'Inline code', defaultKey: 'Mod-e' },
    { id: 'inlineMath', label: 'Inline math', defaultKey: 'Mod-m' },
    { id: 'toggleComment', label: 'Toggle line comment', defaultKey: 'Mod-/' },
    { id: 'find', label: 'Find', defaultKey: 'Mod-f' },
    { id: 'replace', label: 'Find and replace', defaultKey: 'Mod-h' },
    { id: 'addComment', label: 'Comment on selection', defaultKey: 'Mod-Alt-m' },
    { id: 'saveVersion', label: 'Save version', defaultKey: 'Mod-s' }
];
