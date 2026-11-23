import { Facet } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import { formatTypst, type FormatScope } from './typst-api';

export const formatScopeFacet = Facet.define<FormatScope, FormatScope | null>({ combine: (scopes) => scopes[0] ?? null });

function changedSpan(before: string, after: string) {
    let start = 0;
    while (start < before.length && start < after.length && before[start] === after[start]) start++;
    let end = 0;
    while (end < before.length - start && end < after.length - start && before[before.length - 1 - end] === after[after.length - 1 - end]) end++;
    return { from: start, to: before.length - end, insert: after.slice(start, after.length - end) };
}

export async function formatDocument(view: EditorView) {
    const scope = view.state.facet(formatScopeFacet);
    if (!scope || view.state.readOnly) return;
    const before = view.state.doc.toString();
    let after: string;
    try {
        after = await formatTypst(before, scope);
    } catch (error) {
        alert((error as Error).message);
        return;
    }
    if (view.state.doc.toString() !== before || after === before) return;
    view.dispatch({ changes: changedSpan(before, after), userEvent: 'format' });
}
