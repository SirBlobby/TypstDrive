import { EditorState, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';
import { highlightSelectionMatches, search } from '@codemirror/search';

const searchPhrases = EditorState.phrases.of({
    Find: 'Find',
    Replace: 'Replace with',
    next: 'Next',
    previous: 'Previous',
    all: 'Select all',
    'match case': 'Match case',
    regexp: 'Regex',
    'by word': 'Whole word',
    replace: 'Replace',
    'replace all': 'Replace all',
    close: 'Close'
});

const searchPanelTheme = EditorView.theme({
    '.cm-panels.cm-panels-top': {
        borderBottom: '1px solid var(--color-line)',
        backgroundColor: 'var(--color-surface-muted)'
    },
    '.cm-panel.cm-search': {
        position: 'relative',
        padding: '10px 44px 4px 12px',
        fontFamily: 'inherit',
        fontSize: '13px',
        color: 'var(--color-ink)',
        backgroundColor: 'var(--color-surface-muted)'
    },
    '.cm-panel.cm-search .cm-textfield': {
        width: '240px',
        margin: '0 6px 6px 0',
        verticalAlign: 'middle',
        padding: '5px 10px',
        fontSize: '13px',
        color: 'var(--color-ink)',
        backgroundColor: 'var(--color-surface)',
        border: '1px solid var(--color-line)',
        borderRadius: '6px',
        outline: 'none',
        transition: 'border-color 120ms, box-shadow 120ms'
    },
    '.cm-panel.cm-search .cm-textfield:focus': {
        borderColor: 'var(--color-accent)',
        boxShadow: '0 0 0 3px var(--color-accent-soft)'
    },
    '.cm-panel.cm-search .cm-button': {
        margin: '0 4px 6px 0',
        verticalAlign: 'middle',
        padding: '5px 12px',
        fontSize: '12px',
        fontWeight: '500',
        color: 'var(--color-ink)',
        backgroundImage: 'none',
        backgroundColor: 'var(--color-surface)',
        border: '1px solid var(--color-line)',
        borderRadius: '6px',
        cursor: 'pointer',
        transition: 'background-color 120ms, border-color 120ms'
    },
    '.cm-panel.cm-search .cm-button:hover': {
        backgroundColor: 'var(--color-surface-sunken)'
    },
    '.cm-panel.cm-search .cm-button:active': {
        backgroundImage: 'none',
        backgroundColor: 'var(--color-accent-soft)',
        borderColor: 'var(--color-accent)'
    },
    '.cm-panel.cm-search label': {
        display: 'inline-flex',
        alignItems: 'center',
        gap: '5px',
        margin: '0 0 6px 4px',
        verticalAlign: 'middle',
        padding: '4px 8px',
        fontSize: '12px',
        color: 'var(--color-ink-muted)',
        borderRadius: '6px',
        cursor: 'pointer',
        userSelect: 'none'
    },
    '.cm-panel.cm-search label:hover': {
        color: 'var(--color-ink)',
        backgroundColor: 'var(--color-surface-sunken)'
    },
    '.cm-panel.cm-search input[type=checkbox]': {
        margin: '0',
        accentColor: 'var(--color-accent)',
        cursor: 'pointer'
    },
    '.cm-panel.cm-search [name=close]': {
        position: 'absolute',
        top: '10px',
        right: '12px',
        width: '26px',
        height: '26px',
        padding: '0',
        fontSize: '18px',
        lineHeight: '26px',
        color: 'var(--color-ink-muted)',
        backgroundColor: 'transparent',
        border: 'none',
        borderRadius: '6px',
        cursor: 'pointer'
    },
    '.cm-panel.cm-search [name=close]:hover': {
        color: 'var(--color-ink)',
        backgroundColor: 'var(--color-surface-sunken)'
    },
    '.cm-searchMatch': {
        backgroundColor: 'color-mix(in srgb, var(--color-accent) 22%, transparent)',
        outline: '1px solid color-mix(in srgb, var(--color-accent) 45%, transparent)',
        borderRadius: '2px'
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
        backgroundColor: 'color-mix(in srgb, var(--color-accent) 45%, transparent)'
    }
});

export function searchExtensions(): Extension[] {
    return [search({ top: true }), highlightSelectionMatches(), searchPhrases, searchPanelTheme];
}
