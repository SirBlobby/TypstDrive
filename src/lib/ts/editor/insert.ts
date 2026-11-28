import type { EditorView } from '@codemirror/view';
import { openSearchPanel } from '@codemirror/search';

export function insertAtCursor(view: EditorView, insertion: string) {
    const selection = view.state.selection.main;
    view.dispatch({
        changes: { from: selection.from, to: selection.to, insert: insertion },
        selection: { anchor: selection.from + insertion.length },
        scrollIntoView: true
    });
    view.focus();
}

export function insertBlock(view: EditorView, block: string) {
    const selection = view.state.selection.main;
    const line = view.state.doc.lineAt(selection.from);
    const needsLeadingBreak = selection.from > line.from;
    insertAtCursor(view, `${needsLeadingBreak ? '\n' : ''}${block}\n`);
}

export function wrapSelection(view: EditorView, prefix: string, suffix: string, placeholder = '') {
    const selection = view.state.selection.main;
    const selectedText = view.state.doc.sliceString(selection.from, selection.to) || placeholder;
    const start = selection.from + prefix.length;
    view.dispatch({
        changes: { from: selection.from, to: selection.to, insert: prefix + selectedText + suffix },
        selection: { anchor: start, head: start + selectedText.length }
    });
    view.focus();
}

export function setHeadingLevel(view: EditorView, level: number) {
    const lines = new Map<number, { from: number; text: string }>();
    for (const range of view.state.selection.ranges) {
        for (let position = range.from; position <= range.to; ) {
            const line = view.state.doc.lineAt(position);
            lines.set(line.number, { from: line.from, text: line.text });
            position = line.to + 1;
        }
    }
    const changes = [...lines.values()].map((line) => {
        const existing = line.text.match(/^\s*=+\s*/);
        const prefix = level > 0 ? `${'='.repeat(level)} ` : '';
        return { from: line.from, to: line.from + (existing ? existing[0].length : 0), insert: prefix };
    });
    view.dispatch({ changes });
    view.focus();
}

export function currentHeadingLevel(view: EditorView | null) {
    if (!view) return 0;
    const line = view.state.doc.lineAt(view.state.selection.main.head);
    const match = line.text.match(/^\s*(=+)\s/);
    return match ? Math.min(match[1].length, 6) : 0;
}

export function insertQuote(view: EditorView) {
    wrapSelection(view, '#quote(block: true)[', ']', 'Quote');
}

export function buildTable(rowCount: number, columnCount: number, withHeader: boolean) {
    const row = (label: string, rowNumber: number) =>
        '  ' + Array.from({ length: columnCount }, (_, column) => `[${label} ${rowNumber}.${column + 1}]`).join(', ') + ',';
    const lines = [`#table(`, `  columns: ${columnCount},`];
    if (withHeader) {
        lines.push('  table.header(' + Array.from({ length: columnCount }, (_, column) => `[*Header ${column + 1}*]`).join(', ') + '),');
    }
    for (let rowNumber = 1; rowNumber <= rowCount; rowNumber++) {
        lines.push(row('Cell', rowNumber));
    }
    lines.push(')');
    return lines.join('\n');
}

export function openFind(view: EditorView) {
    openSearchPanel(view);
}

export function openReplace(view: EditorView) {
    openSearchPanel(view);
    requestAnimationFrame(() => {
        const replaceField = view.dom.querySelector<HTMLInputElement>('.cm-search input[name="replace"]');
        replaceField?.focus();
    });
}
