import type { EditorView } from '@codemirror/view';
import { insertAtCursor, wrapSelection } from '../editor-insert';
import { buildLatexTable, figureSnippet, insertList, setSectionLevel, wrapEnvironment } from '../editor-latex-actions';
import { insertDialogStore } from '../insert-dialog';
import type { SlashCommand } from './slash-command-types';

function wrap(prefix: string, suffix: string, placeholder: string) {
    return (view: EditorView) => wrapSelection(view, prefix, suffix, placeholder);
}

function environment(name: string, placeholder: string) {
    return (view: EditorView) => wrapEnvironment(view, name, placeholder);
}

function section(level: number) {
    return (view: EditorView) => setSectionLevel(view, level);
}

function insert(text: string) {
    return (view: EditorView) => insertAtCursor(view, text);
}

export const latexSlashCommands: SlashCommand[] = [
    { id: 'section', label: 'Section', description: 'Large section heading', icon: 'mdi:format-header-1', group: 'Text', keywords: ['heading', 'h1'], run: section(1) },
    { id: 'subsection', label: 'Subsection', description: 'Medium section heading', icon: 'mdi:format-header-2', group: 'Text', keywords: ['heading', 'h2'], run: section(2) },
    { id: 'subsubsection', label: 'Subsubsection', description: 'Small section heading', icon: 'mdi:format-header-3', group: 'Text', keywords: ['heading', 'h3'], run: section(3) },
    { id: 'normal-text', label: 'Normal text', description: 'Turn a section back into text', icon: 'mdi:format-text', group: 'Text', keywords: ['paragraph', 'plain'], run: section(0) },
    { id: 'bold', label: 'Bold', description: 'Strong text', icon: 'mdi:format-bold', group: 'Text', keywords: ['textbf'], shortcut: 'bold', run: wrap('\\textbf{', '}', 'bold') },
    { id: 'italic', label: 'Italic', description: 'Emphasized text', icon: 'mdi:format-italic', group: 'Text', keywords: ['textit', 'emph'], shortcut: 'italic', run: wrap('\\textit{', '}', 'italic') },
    { id: 'underline', label: 'Underline', description: 'Underlined text', icon: 'mdi:format-underline', group: 'Text', keywords: [], run: wrap('\\underline{', '}', 'text') },
    { id: 'inline-code', label: 'Monospace', description: 'Typewriter text', icon: 'mdi:code-tags', group: 'Text', keywords: ['texttt', 'code'], shortcut: 'code', run: wrap('\\texttt{', '}', 'code') },
    { id: 'verbatim', label: 'Verbatim block', description: 'Block of code shown as is', icon: 'mdi:code-braces', group: 'Text', keywords: ['code', 'raw'], run: environment('verbatim', 'code') },
    { id: 'quote', label: 'Quote', description: 'Block quotation', icon: 'mdi:format-quote-close', group: 'Text', keywords: ['blockquote'], run: environment('quote', 'Quote') },
    { id: 'bullet-list', label: 'Bullet list', description: 'itemize environment', icon: 'mdi:format-list-bulleted', group: 'Lists', keywords: ['itemize', 'unordered'], run: (view) => insertList(view, 'itemize') },
    { id: 'numbered-list', label: 'Numbered list', description: 'enumerate environment', icon: 'mdi:format-list-numbered', group: 'Lists', keywords: ['enumerate', 'ordered'], run: (view) => insertList(view, 'enumerate') },
    {
        id: 'image',
        label: 'Image',
        description: 'Search your images or upload one',
        icon: 'mdi:image-outline',
        group: 'Insert',
        keywords: ['picture', 'includegraphics', 'upload'],
        run: (view, context) => (context.insertDialogs ? insertDialogStore.set('image') : wrapSelection(view, '\\includegraphics[width=0.8\\linewidth]{', '}', 'file.png'))
    },
    { id: 'figure', label: 'Figure', description: 'Image with a caption', icon: 'mdi:image-frame', group: 'Insert', keywords: ['image', 'includegraphics'], run: insert(figureSnippet('file.png')) },
    { id: 'table', label: 'Table', description: 'tabular environment', icon: 'mdi:table', group: 'Insert', keywords: ['tabular', 'grid'], run: insert(buildLatexTable(3, 3, true) + '\n') },
    { id: 'link', label: 'Link', description: 'Clickable web link', icon: 'mdi:link-variant', group: 'Insert', keywords: ['href', 'url'], run: wrap('\\href{https://}{', '}', 'text') },
    { id: 'footnote', label: 'Footnote', description: 'Note at the bottom of the page', icon: 'mdi:format-superscript', group: 'Insert', keywords: ['note'], run: wrap('\\footnote{', '}', 'Footnote text') },
    { id: 'citation', label: 'Citation', description: 'Cite a bibliography entry', icon: 'ph:quotes', group: 'Insert', keywords: ['cite'], run: wrap('\\cite{', '}', 'key') },
    { id: 'label', label: 'Label', description: 'Name something to refer to it', icon: 'mdi:tag-outline', group: 'Insert', keywords: ['anchor'], run: wrap('\\label{', '}', 'sec:label') },
    { id: 'reference', label: 'Reference', description: 'Refer to a label', icon: 'mdi:arrow-right-bottom', group: 'Insert', keywords: ['ref'], run: wrap('\\ref{', '}', 'sec:label') },
    { id: 'inline-math', label: 'Inline math', description: 'Math inside a line', icon: 'mdi:sigma', group: 'Math', keywords: ['equation', 'formula'], shortcut: 'inlineMath', run: wrap('$', '$', 'x = y') },
    { id: 'equation', label: 'Equation', description: 'Numbered equation block', icon: 'mdi:math-integral', group: 'Math', keywords: ['display', 'math block'], run: environment('equation', 'x = y') },
    { id: 'fraction', label: 'Fraction', description: 'Numerator over denominator', icon: 'mdi:division', group: 'Math', keywords: ['frac', 'divide'], run: wrap('\\frac{', '}{b}', 'a') },
    { id: 'page-break', label: 'Page break', description: 'Start a new page', icon: 'mdi:file-document-arrow-right-outline', group: 'Layout', keywords: ['newpage'], run: insert('\\newpage\n') },
    { id: 'divider', label: 'Divider', description: 'Horizontal line', icon: 'mdi:minus', group: 'Layout', keywords: ['hrule', 'line', 'rule'], run: insert('\\noindent\\rule{\\linewidth}{0.4pt}\n') },
    { id: 'outline', label: 'Table of contents', description: 'List of sections', icon: 'mdi:table-of-contents', group: 'Layout', keywords: ['toc', 'contents'], run: insert('\\tableofcontents\n') }
];
