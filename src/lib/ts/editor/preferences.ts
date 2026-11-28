import type { Extension } from '@codemirror/state';
import { EditorView, lineNumbers } from '@codemirror/view';
import { minimap } from './minimap';
import { findEditorFont } from './fonts';
import { spellCheckExtension } from './spellcheck';
import type { EditorSettings } from './settings';

export function editorPreferenceExtensions(settings: EditorSettings): Extension {
    const font = findEditorFont(settings.fontFamily);
    return [
        settings.lineNumbers ? lineNumbers() : [],
        settings.minimap ? minimap(settings.minimapSize) : [],
        spellCheckExtension(settings.spellCheck, settings.spellCheckLanguage),
        EditorView.theme({
            '&': { fontSize: `${settings.fontSize}px` },
            '.cm-scroller, .cm-content, .cm-gutters': { fontFamily: font.stack }
        })
    ];
}
