import { get, writable } from 'svelte/store';
import { announceReviewChange } from './live';
import type * as Y from 'yjs';
import type { PersonSummary } from '../sharing/people';
import { resolveRange, type AnchoredRange } from './anchors';
import { createUrl, itemSource, itemUrl, listUrl, type ReviewSource, type ReviewTarget } from './target';

export type SuggestionStatus = 'pending' | 'accepted' | 'rejected' | 'withdrawn';

export interface SuggestionView {
    id: string;
    document_id?: string;
    file_id?: string;
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

export interface SuggestionContent extends AnchoredRange {
    original_text: string;
    replacement_text: string;
}

export const suggestionsStore = writable<SuggestionView[]>([]);
export const suggestingModeStore = writable(false);
export const activeSuggestionStore = writable<string | null>(null);

async function errorText(res: Response, fallback: string) {
    return (await res.text()) || fallback;
}

export async function loadSuggestions(target: ReviewTarget) {
    const res = await fetch(listUrl(target, 'suggestions'));
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load suggestions.'));
    suggestionsStore.set(await res.json());
}

export async function createSuggestion(target: ReviewTarget, content: SuggestionContent): Promise<SuggestionView> {
    const res = await fetch(createUrl(target, 'suggestions'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...content, note: null })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to save the suggestion.'));
    return res.json();
}

const CONFLICT_STATUS = 409;

export class SuggestionConflictError extends Error {}

interface SuggestionChanges {
    status?: SuggestionStatus;
    content?: SuggestionContent;
    expected_replacement_text?: string;
}

async function patchSuggestion(target: ReviewTarget, suggestionId: string, changes: SuggestionChanges) {
    const res = await fetch(itemUrl(target, 'suggestions', suggestionId), {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(changes)
    });
    if (res.status === CONFLICT_STATUS) throw new SuggestionConflictError(await errorText(res, 'This suggestion changed or was already handled.'));
    if (!res.ok) throw new Error(await errorText(res, 'Failed to update the suggestion.'));
}

export function updateSuggestionContent(target: ReviewTarget, suggestionId: string, content: SuggestionContent) {
    return patchSuggestion(target, suggestionId, { content });
}

export async function setSuggestionStatus(target: ReviewTarget, suggestionId: string, status: SuggestionStatus) {
    await patchSuggestion(target, suggestionId, { status });
    announceReviewChange();
    await loadSuggestions(target);
}
export function currentSuggestionRange(doc: Y.Doc, suggestion: SuggestionView) {
    return resolveRange(doc, suggestion);
}

async function latestVersion(target: ReviewTarget, suggestionId: string) {
    await loadSuggestions(target);
    const latest = get(suggestionsStore).find((candidate) => candidate.id === suggestionId);
    if (!latest || latest.status !== 'pending') throw new SuggestionConflictError('This suggestion was already handled.');
    return latest;
}

export async function acceptSuggestion(target: ReviewTarget, suggestion: SuggestionView) {
    const latest = await latestVersion(target, suggestion.id);
    const source = itemSource(target, latest);
    if (!source) throw new Error('The file this suggestion belongs to is not open.');
    if (!currentSuggestionRange(source.doc, latest)) throw new Error('The text this suggestion refers to no longer exists.');

    await patchSuggestion(target, latest.id, { status: 'accepted', expected_replacement_text: latest.replacement_text });

    const range = currentSuggestionRange(source.doc, latest);
    if (range) {
        source.doc.transact(() => {
            if (range.to > range.from) source.text.delete(range.from, range.to - range.from);
            if (latest.replacement_text) source.text.insert(range.from, latest.replacement_text);
        });
    }
    announceReviewChange();
    await loadSuggestions(target);
}

export function rejectSuggestion(target: ReviewTarget, suggestion: SuggestionView) {
    return setSuggestionStatus(target, suggestion.id, 'rejected');
}

export function withdrawSuggestion(target: ReviewTarget, suggestion: SuggestionView) {
    return setSuggestionStatus(target, suggestion.id, 'withdrawn');
}

export function textChangedSince(source: ReviewSource, suggestion: SuggestionView) {
    const range = currentSuggestionRange(source.doc, suggestion);
    if (!range) return true;
    return source.text.toString().slice(range.from, range.to) !== suggestion.original_text;
}
