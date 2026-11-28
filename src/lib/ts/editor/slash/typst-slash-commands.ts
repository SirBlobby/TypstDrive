import type { EditorView } from '@codemirror/view';
import { buildTable, insertBlock, insertQuote, setHeadingLevel, wrapSelection } from '../insert';
import { insertDialogStore, type InsertDialog } from '../insert-dialog';
import type { SlashCommand, SlashContext } from './slash-command-types';

function wrap(prefix: string, suffix: string, placeholder: string) {
    return (view: EditorView) => wrapSelection(view, prefix, suffix, placeholder);
}

function block(text: string) {
    return (view: EditorView) => insertBlock(view, text);
}

function heading(level: number) {
    return (view: EditorView) => setHeadingLevel(view, level);
}

function dialogOr(dialog: InsertDialog, fallback: (view: EditorView) => void) {
    return (view: EditorView, context: SlashContext) => (context.insertDialogs ? insertDialogStore.set(dialog) : fallback(view));
}

const withDialogs = (context: SlashContext) => context.insertDialogs;

const FIGURE_TEMPLATE = '#figure(\n  image("image.png", width: 80%),\n  caption: [Caption],\n)';

export const typstSlashCommands: SlashCommand[] = [
    { id: 'heading-1', label: 'Heading 1', description: 'Large section heading', icon: 'mdi:format-header-1', group: 'Text', keywords: ['h1', 'title'], run: heading(1) },
    { id: 'heading-2', label: 'Heading 2', description: 'Medium section heading', icon: 'mdi:format-header-2', group: 'Text', keywords: ['h2', 'subtitle'], run: heading(2) },
    { id: 'heading-3', label: 'Heading 3', description: 'Small section heading', icon: 'mdi:format-header-3', group: 'Text', keywords: ['h3'], run: heading(3) },
    { id: 'normal-text', label: 'Normal text', description: 'Turn a heading back into text', icon: 'mdi:format-text', group: 'Text', keywords: ['paragraph', 'plain'], run: heading(0) },
    { id: 'bold', label: 'Bold', description: 'Strong text', icon: 'mdi:format-bold', group: 'Text', keywords: ['strong'], shortcut: 'bold', run: wrap('*', '*', 'bold') },
    { id: 'italic', label: 'Italic', description: 'Emphasized text', icon: 'mdi:format-italic', group: 'Text', keywords: ['emphasis'], shortcut: 'italic', run: wrap('_', '_', 'italic') },
    { id: 'underline', label: 'Underline', description: 'Underlined text', icon: 'mdi:format-underline', group: 'Text', keywords: [], run: wrap('#underline[', ']', 'text') },
    { id: 'strike', label: 'Strikethrough', description: 'Crossed out text', icon: 'mdi:format-strikethrough', group: 'Text', keywords: ['strike'], run: wrap('#strike[', ']', 'text') },
    { id: 'highlight', label: 'Highlight', description: 'Marked text', icon: 'mdi:marker', group: 'Text', keywords: ['mark'], run: wrap('#highlight[', ']', 'text') },
    { id: 'inline-code', label: 'Inline code', description: 'Monospaced code in a line', icon: 'mdi:code-tags', group: 'Text', keywords: ['raw'], shortcut: 'code', run: wrap('`', '`', 'code') },
    { id: 'code-block', label: 'Code block', description: 'Block of monospaced code', icon: 'mdi:code-braces', group: 'Text', keywords: ['raw', 'snippet'], run: wrap('```\n', '\n```', 'code') },
    { id: 'quote', label: 'Quote', description: 'Block quotation', icon: 'mdi:format-quote-close', group: 'Text', keywords: ['blockquote'], run: insertQuote },
    { id: 'bullet-list', label: 'Bullet list', description: 'List with dots', icon: 'mdi:format-list-bulleted', group: 'Lists', keywords: ['unordered', 'ul'], run: wrap('- ', '', 'List item') },
    { id: 'numbered-list', label: 'Numbered list', description: 'List with numbers', icon: 'mdi:format-list-numbered', group: 'Lists', keywords: ['ordered', 'ol', 'enum'], run: wrap('+ ', '', 'Numbered item') },
    { id: 'term-list', label: 'Term list', description: 'Terms with descriptions', icon: 'mdi:format-list-text', group: 'Lists', keywords: ['definition', 'glossary'], run: wrap('/ ', ': Description', 'Term') },
    { id: 'image', label: 'Image', description: 'Search your images or upload one', icon: 'mdi:image-outline', group: 'Insert', keywords: ['picture', 'photo', 'upload'], run: dialogOr('image', wrap('#image("', '")', 'image.png')) },
    { id: 'table', label: 'Table', description: 'Rows and columns', icon: 'mdi:table', group: 'Insert', keywords: ['grid'], run: dialogOr('table', block(buildTable(3, 3, true))) },
    { id: 'figure', label: 'Figure', description: 'Image with a caption', icon: 'mdi:image-frame', group: 'Insert', keywords: ['caption'], run: block(FIGURE_TEMPLATE) },
    { id: 'link', label: 'Link', description: 'Clickable web link', icon: 'mdi:link-variant', group: 'Insert', keywords: ['url', 'href'], run: wrap('#link("https://")[', ']', 'link text') },
    { id: 'footnote', label: 'Footnote', description: 'Note at the bottom of the page', icon: 'mdi:format-superscript', group: 'Insert', keywords: ['note'], run: wrap('#footnote[', ']', 'Footnote') },
    { id: 'citation', label: 'Citation', description: 'Cite a bibliography entry', icon: 'ph:quotes', group: 'Insert', keywords: ['cite', 'reference'], run: wrap('@', '', 'key') },
    { id: 'label', label: 'Label', description: 'Name something to refer to it', icon: 'mdi:tag-outline', group: 'Insert', keywords: ['anchor', 'ref'], run: wrap('<', '>', 'label') },
    { id: 'symbol', label: 'Symbol', description: 'Pick a special character', icon: 'mdi:omega', group: 'Insert', keywords: ['character', 'greek'], isAvailable: withDialogs, run: () => insertDialogStore.set('symbol') },
    { id: 'emoji', label: 'Emoji', description: 'Pick an emoji', icon: 'mdi:emoticon-happy-outline', group: 'Insert', keywords: ['smiley'], isAvailable: withDialogs, run: () => insertDialogStore.set('emoji') },
    { id: 'document', label: 'Embed document', description: 'Include another document', icon: 'mdi:file-document-outline', group: 'Insert', keywords: ['include'], isAvailable: (context) => context.embedDocuments, run: () => insertDialogStore.set('document') },
    { id: 'date', label: 'Today\'s date', description: 'Date when the document compiles', icon: 'mdi:calendar-today', group: 'Insert', keywords: ['today', 'time'], run: wrap('#datetime.today().display()', '', '') },
    { id: 'inline-math', label: 'Inline math', description: 'Math inside a line', icon: 'mdi:sigma', group: 'Math', keywords: ['equation', 'formula'], shortcut: 'inlineMath', run: wrap('$', '$', 'x = y') },
    { id: 'block-math', label: 'Math block', description: 'Centered equation', icon: 'mdi:math-integral', group: 'Math', keywords: ['equation', 'display'], run: wrap('$ ', ' $', 'x = y') },
    { id: 'fraction', label: 'Fraction', description: 'Numerator over denominator', icon: 'mdi:division', group: 'Math', keywords: ['frac', 'divide'], run: wrap('$frac(', ', b)$', 'a') },
    { id: 'page-break', label: 'Page break', description: 'Start a new page', icon: 'mdi:file-document-arrow-right-outline', group: 'Layout', keywords: ['new page', 'pagebreak'], run: block('#pagebreak()') },
    { id: 'column-break', label: 'Column break', description: 'Move to the next column', icon: 'mdi:view-column-outline', group: 'Layout', keywords: ['colbreak'], run: block('#colbreak()') },
    { id: 'divider', label: 'Divider', description: 'Horizontal line', icon: 'mdi:minus', group: 'Layout', keywords: ['line', 'rule', 'hr', 'separator'], run: block('#line(length: 100%)') },
    { id: 'outline', label: 'Table of contents', description: 'List of headings', icon: 'mdi:table-of-contents', group: 'Layout', keywords: ['toc', 'outline', 'contents'], run: block('#outline()') },
    { id: 'bibliography', label: 'Bibliography', description: 'List of references', icon: 'mdi:bookshelf', group: 'Layout', keywords: ['references', 'sources'], run: block('#bibliography("references.bib")') }
];
