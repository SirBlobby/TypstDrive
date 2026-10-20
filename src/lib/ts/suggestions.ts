import { writable } from 'svelte/store';
import type * as Y from 'yjs';
import type { PersonSummary } from './people';
import { resolveRange, type AnchoredRange } from './document-anchors';

export type SuggestionStatus = 'pending' | 'accepted' | 'rejected' | 'withdrawn';

export interface SuggestionView {
    id: string;
    document_id: string;
    user_id: string;
    author: PersonSummary | null;
    anchor_start: string;
    anchor_end: string;
    original_text: string;
    replacement_text: string;
    note: string | null;
    status: SuggestionStatus;
    resolved_by: PersonSummary | null;
    resolved_at: string | null;
    created_at: string;
}

export interface SuggestionDraft extends AnchoredRange {
    from: number;
    to: number;
    original_text: string;
    replacement_text: string;
}

export const suggestionsStore = writable<SuggestionView[]>([]);
export const suggestionDraftStore = writable<SuggestionDraft | null>(null);
export const suggestingModeStore = writable(false);
export const activeSuggestionStore = writable<string | null>(null);

async function errorText(res: Response, fallback: string) {
    return (await res.text()) || fallback;
}

export async function loadSuggestions(docId: string) {
    const res = await fetch(`/api/docs/${docId}/suggestions`);
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load suggestions.'));
    suggestionsStore.set(await res.json());
}

export async function submitSuggestion(docId: string, draft: SuggestionDraft, note: string) {
    const res = await fetch(`/api/docs/${docId}/suggestions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            anchor_start: draft.anchor_start,
            anchor_end: draft.anchor_end,
            original_text: draft.original_text,
            replacement_text: draft.replacement_text,
            note: note || null
        })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to save the suggestion.'));
    await loadSuggestions(docId);
}

async function setStatus(docId: string, suggestionId: string, status: SuggestionStatus) {
    const res = await fetch(`/api/suggestions/${suggestionId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to update the suggestion.'));
    await loadSuggestions(docId);
}

export function currentSuggestionRange(doc: Y.Doc, suggestion: SuggestionView) {
    return resolveRange(doc, suggestion);
}

export async function acceptSuggestion(docId: string, doc: Y.Doc, text: Y.Text, suggestion: SuggestionView) {
    const range = currentSuggestionRange(doc, suggestion);
    if (!range) throw new Error('The text this suggestion refers to no longer exists.');
    doc.transact(() => {
        if (range.to > range.from) text.delete(range.from, range.to - range.from);
        if (suggestion.replacement_text) text.insert(range.from, suggestion.replacement_text);
    });
    await setStatus(docId, suggestion.id, 'accepted');
}

export function rejectSuggestion(docId: string, suggestion: SuggestionView) {
    return setStatus(docId, suggestion.id, 'rejected');
}

export function withdrawSuggestion(docId: string, suggestion: SuggestionView) {
    return setStatus(docId, suggestion.id, 'withdrawn');
}

export function textChangedSince(doc: Y.Doc, text: Y.Text, suggestion: SuggestionView) {
    const range = currentSuggestionRange(doc, suggestion);
    if (!range) return true;
    return text.toString().slice(range.from, range.to) !== suggestion.original_text;
}
