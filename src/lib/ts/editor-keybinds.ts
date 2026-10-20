import { Prec, type Extension } from '@codemirror/state';
import { keymap, type EditorView, type KeyBinding } from '@codemirror/view';
import { get } from 'svelte/store';
import { keybindsStore } from './keybinds/keybinds-store';
import type { KeybindActionId } from './keybinds/keybind-actions';
import { openFind, openReplace, wrapSelection } from './editor-insert';
import { toggleLineComment } from './editor-comments';

type EditorCommand = (view: EditorView) => boolean;

function wrapping(prefix: string, suffix: string, placeholder: string): EditorCommand {
    return (view) => {
        if (view.state.readOnly) return false;
        wrapSelection(view, prefix, suffix, placeholder);
        return true;
    };
}

export function editorKeybindExtension(commentMarker: string, isTypst: boolean): Extension {
    const keybinds = get(keybindsStore);

    const commands: Partial<Record<KeybindActionId, EditorCommand>> = {
        toggleComment: (view) => toggleLineComment(view, commentMarker),
        find: (view) => {
            openFind(view);
            return true;
        },
        replace: (view) => {
            openReplace(view);
            return true;
        }
    };

    if (isTypst) {
        commands.bold = wrapping('*', '*', 'bold');
        commands.italic = wrapping('_', '_', 'italic');
        commands.code = wrapping('`', '`', 'code');
        commands.inlineMath = wrapping('$ ', ' $', 'x = y');
    }

    const bindings: KeyBinding[] = Object.entries(commands)
        .filter(([id]) => keybinds[id as KeybindActionId])
        .map(([id, run]) => ({ key: keybinds[id as KeybindActionId], run, preventDefault: true }));

    return Prec.high(keymap.of(bindings));
}
