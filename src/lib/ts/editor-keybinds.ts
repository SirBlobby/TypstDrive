import { Prec, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { get } from 'svelte/store';
import { keybindsStore } from './keybinds/keybinds-store';
import type { KeybindActionId } from './keybinds/keybind-actions';
import { matchesKeybind } from './keybinds/key-format';
import { formatDocument } from './editor-format';
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

export function editorKeybindExtension(commentMarker: string, isTypst: boolean, isLatex = false): Extension {
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
        commands.formatDocument = (view) => {
            formatDocument(view);
            return true;
        };
    }

    if (isLatex) {
        commands.bold = wrapping('\\textbf{', '}', 'bold');
        commands.italic = wrapping('\\textit{', '}', 'italic');
        commands.code = wrapping('\\texttt{', '}', 'code');
        commands.inlineMath = wrapping('$', '$', 'x = y');
    }

    const bindings = Object.entries(commands).filter(([id]) => keybinds[id as KeybindActionId]);

    // Matched by hand rather than with keymap.of: CodeMirror's keymap skips Option-only combos on macOS, where they type characters such as Ï.
    return Prec.high(
        EditorView.domEventHandlers({
            keydown(event, view) {
                const match = bindings.find(([id]) => matchesKeybind(keybinds[id as KeybindActionId], event));
                if (!match) return false;
                // Like keymap's preventDefault: a matched shortcut never falls through to the browser, even when its command declines.
                match[1](view);
                event.preventDefault();
                return true;
            }
        })
    );
}
