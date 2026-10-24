import type { Extension } from '@codemirror/state';
import { latex } from 'codemirror-lang-latex';

export function latexEditorExtensions(): Extension[] {
    return [
        latex({
            autoCloseTags: true,
            autoCloseBrackets: false,
            enableLinting: true,
            enableTooltips: true,
            enableAutocomplete: true,
            fileName: 'main.tex'
        })
    ];
}
