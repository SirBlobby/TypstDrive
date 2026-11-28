import type { EditorView } from '@codemirror/view';

export interface UploadedDocumentFile {
    filename: string;
    font_family?: string;
}

export async function uploadDocumentFile(docId: string, file: File): Promise<UploadedDocumentFile | null> {
    const formData = new FormData();
    formData.append('file', file);
    const response = await fetch(`/api/docs/${docId}/files`, { method: 'POST', body: formData });
    const data = await response.json();
    return data.filename ? data : null;
}

export function replaceSelection(view: EditorView, insertion: string) {
    const selection = view.state.selection.main;
    view.dispatch({
        changes: { from: selection.from, to: selection.to, insert: insertion },
        selection: { anchor: selection.from + insertion.length }
    });
    view.focus();
}
