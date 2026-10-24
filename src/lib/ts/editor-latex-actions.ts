import type { EditorView } from '@codemirror/view';
import { insertAtCursor, wrapSelection } from './editor-insert';

export const latexSectionCommands = ['section', 'subsection', 'subsubsection', 'paragraph', 'subparagraph'];

export const latexHeadingOptions = [
    { level: 0, label: 'Normal text' },
    { level: 1, label: 'Section' },
    { level: 2, label: 'Subsection' },
    { level: 3, label: 'Subsubsection' },
    { level: 4, label: 'Paragraph' },
    { level: 5, label: 'Subparagraph' }
];

const SECTION_PATTERN = /^(\s*)\\(section|subsection|subsubsection|paragraph|subparagraph)\*?\{(.*)\}\s*$/;

export function currentSectionLevel(view: EditorView | null) {
    if (!view) return 0;
    const line = view.state.doc.lineAt(view.state.selection.main.head);
    const match = line.text.match(SECTION_PATTERN);
    return match ? latexSectionCommands.indexOf(match[2]) + 1 : 0;
}

export function setSectionLevel(view: EditorView, level: number) {
    const line = view.state.doc.lineAt(view.state.selection.main.head);
    const match = line.text.match(SECTION_PATTERN);
    const indent = match ? match[1] : (line.text.match(/^\s*/)?.[0] ?? '');
    const title = match ? match[3] : line.text.trim();
    const command = latexSectionCommands[level - 1];
    const replacement = command ? `${indent}\\${command}{${title || 'Title'}}` : `${indent}${title}`;

    view.dispatch({
        changes: { from: line.from, to: line.to, insert: replacement },
        selection: { anchor: line.from + replacement.length }
    });
    view.focus();
}

export function wrapEnvironment(view: EditorView, environment: string, placeholder: string) {
    wrapSelection(view, `\\begin{${environment}}\n  `, `\n\\end{${environment}}`, placeholder);
}

export function insertList(view: EditorView, environment: 'itemize' | 'enumerate') {
    const selection = view.state.selection.main;
    const startLine = view.state.doc.lineAt(selection.from);
    const endLine = view.state.doc.lineAt(selection.to);
    const selectedLines = selection.empty
        ? []
        : view.state.doc.sliceString(startLine.from, endLine.to).split('\n').filter((text) => text.trim() !== '');

    const items = (selectedLines.length > 0 ? selectedLines : ['List item'])
        .map((text) => `  \\item ${text.trim().replace(/^\\item\s*/, '')}`)
        .join('\n');
    const block = `\\begin{${environment}}\n${items}\n\\end{${environment}}`;
    const from = selection.empty ? selection.from : startLine.from;
    const to = selection.empty ? selection.to : endLine.to;

    view.dispatch({
        changes: { from, to, insert: block },
        selection: { anchor: from + block.length }
    });
    view.focus();
}

export function buildLatexTable(rowCount: number, columnCount: number, withHeader: boolean) {
    const row = (label: string, rowNumber: number) =>
        '    ' + Array.from({ length: columnCount }, (_, column) => `${label} ${rowNumber}.${column + 1}`).join(' & ') + ' \\\\';
    const lines = ['\\begin{table}[h]', '  \\centering', `  \\begin{tabular}{${'l'.repeat(columnCount)}}`, '    \\hline'];
    if (withHeader) {
        lines.push('    ' + Array.from({ length: columnCount }, (_, column) => `\\textbf{Header ${column + 1}}`).join(' & ') + ' \\\\');
        lines.push('    \\hline');
    }
    for (let rowNumber = 1; rowNumber <= rowCount; rowNumber++) {
        lines.push(row('Cell', rowNumber));
    }
    lines.push('    \\hline', '  \\end{tabular}', '  \\caption{Caption}', '\\end{table}');
    return lines.join('\n');
}

export function figureSnippet(fileName: string) {
    return `\\begin{figure}[h]\n  \\centering\n  \\includegraphics[width=0.8\\linewidth]{${fileName}}\n  \\caption{Caption}\n  \\label{fig:label}\n\\end{figure}\n`;
}

export interface LatexInsertOption {
    label: string;
    icon: string;
    run: (view: EditorView) => void;
}

export const latexInsertOptions: LatexInsertOption[] = [
    { label: 'Figure', icon: 'mdi:image-outline', run: (view) => insertAtCursor(view, figureSnippet('file.png')) },
    { label: 'Table', icon: 'mdi:table', run: (view) => insertAtCursor(view, buildLatexTable(3, 3, true) + '\n') },
    { label: 'Link', icon: 'mdi:link-variant', run: (view) => wrapSelection(view, '\\href{https://}{', '}', 'text') },
    { label: 'Footnote', icon: 'mdi:format-superscript', run: (view) => wrapSelection(view, '\\footnote{', '}', 'Footnote text') },
    { label: 'Label', icon: 'mdi:tag-outline', run: (view) => wrapSelection(view, '\\label{', '}', 'sec:label') },
    { label: 'Reference', icon: 'mdi:arrow-right-bottom', run: (view) => wrapSelection(view, '\\ref{', '}', 'sec:label') },
    { label: 'Citation', icon: 'mdi:book-open-variant', run: (view) => wrapSelection(view, '\\cite{', '}', 'key') },
    { label: 'Verbatim block', icon: 'mdi:code-braces-box', run: (view) => wrapEnvironment(view, 'verbatim', 'code') },
    { label: 'Table of contents', icon: 'mdi:format-list-text', run: (view) => insertAtCursor(view, '\\tableofcontents\n') },
    { label: 'Page break', icon: 'mdi:page-next-outline', run: (view) => insertAtCursor(view, '\\newpage\n') }
];
