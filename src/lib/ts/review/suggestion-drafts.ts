import { get, writable } from 'svelte/store';
import type { EditorState } from '@codemirror/state';
import { userStore } from '../account/auth';
import { anchorRange, resolveRange, type AnchoredRange } from './anchors';
import { activeSource, currentReviewTarget, inActiveFile, type ReviewSource, type ReviewTarget } from './target';
import {
    createSuggestion,
    loadSuggestions,
    setSuggestionStatus,
    SuggestionConflictError,
    suggestionsStore,
    updateSuggestionContent,
    type SuggestionView
} from './suggestions';
import { isEmptyDraft, mergeEdit, startDraft, touches, type DraftRange } from './suggestion-merge';
import type { CapturedEdit, SuggestingHandlers } from '../editor/suggesting-input';
import type { Awareness } from 'y-protocols/awareness';
import { announceReviewChange, shareDraft } from './live';

export interface LiveDraft extends AnchoredRange {
    replacement: string;
    caret: number;
    serverId: string | null;
}

interface DraftOwner {
    draft: LiveDraft;
    target: ReviewTarget;
    source: ReviewSource;
}

const SAVE_DELAY_MS = 800;
export const LIVE_DRAFT_ID = 'live-draft';

export const liveDraftStore = writable<LiveDraft | null>(null);

let current: DraftOwner | null = null;
let saveTimer: ReturnType<typeof setTimeout> | undefined;
let saveQueue: Promise<void> = Promise.resolve();

let sharedOn: Awareness | null = null;

function shareCurrentDraft() {
    const awareness = current?.source.awareness ?? null;
    if (sharedOn && sharedOn !== awareness) shareDraft(sharedOn, null);
    const draft = current?.draft;
    shareDraft(awareness, draft ? { anchor_start: draft.anchor_start, anchor_end: draft.anchor_end, replacement: draft.replacement, serverId: draft.serverId } : null);
    sharedOn = awareness;
}

function publish() {
    const snapshot = current ? { ...current.draft } : null;
    queueMicrotask(() => {
        liveDraftStore.set(snapshot);
        shareCurrentDraft();
    });
}

function resolveDraft(owner: DraftOwner): DraftRange | null {
    const range = resolveRange(owner.source.doc, owner.draft);
    return range ? { ...range, replacement: owner.draft.replacement, caret: owner.draft.caret } : null;
}

async function persist(owner: DraftOwner) {
    const { draft, target, source } = owner;
    const range = resolveRange(source.doc, draft);
    if (!range) return;
    const originalText = source.text.toString().slice(range.from, range.to);

    if (originalText === draft.replacement) {
        if (draft.serverId) await setSuggestionStatus(target, draft.serverId, 'withdrawn');
        draft.serverId = null;
        return;
    }

    const content = {
        anchor_start: draft.anchor_start,
        anchor_end: draft.anchor_end,
        original_text: originalText,
        replacement_text: draft.replacement
    };
    if (draft.serverId) {
        await updateSuggestionContent(target, draft.serverId, content);
    } else {
        draft.serverId = (await createSuggestion(target, content)).id;
        if (current?.draft === draft) publish();
    }
    announceReviewChange();
    await loadSuggestions(target);
}

function dropDraft(draft: LiveDraft) {
    if (current?.draft !== draft) return;
    clearTimeout(saveTimer);
    current = null;
    publish();
}

async function persistSafely(owner: DraftOwner) {
    try {
        await persist(owner);
    } catch (error) {
        if (!(error instanceof SuggestionConflictError)) throw error;
        dropDraft(owner.draft);
        await loadSuggestions(owner.target);
    }
}

function queueSave(owner: DraftOwner) {
    saveQueue = saveQueue.then(() => persistSafely(owner)).catch((error) => console.error('Failed to save suggestion', error));
}

suggestionsStore.subscribe((suggestions) => {
    const serverId = current?.draft.serverId;
    if (!current || !serverId) return;
    const saved = suggestions.find((suggestion) => suggestion.id === serverId);
    if (saved && saved.status !== 'pending') dropDraft(current.draft);
});

function queueWithdraw(target: ReviewTarget, suggestionId: string) {
    saveQueue = saveQueue.then(() => setSuggestionStatus(target, suggestionId, 'withdrawn')).catch(() => {});
}

function scheduleSave() {
    clearTimeout(saveTimer);
    const owner = current;
    if (owner) saveTimer = setTimeout(() => queueSave(owner), SAVE_DELAY_MS);
}

export function endLiveDraft() {
    clearTimeout(saveTimer);
    if (current) queueSave(current);
    current = null;
    publish();
}

export function isLiveDraftSuggestion(suggestionId: string) {
    return suggestionId === LIVE_DRAFT_ID || (!!current && current.draft.serverId === suggestionId);
}

function activeContext() {
    const target = currentReviewTarget();
    const source = target ? activeSource(target) : null;
    return target && source ? { target, source } : null;
}

function ownPendingSuggestions(target: ReviewTarget): SuggestionView[] {
    const userId = get(userStore)?.id;
    if (!userId) return [];
    return get(suggestionsStore).filter(
        (suggestion) => suggestion.status === 'pending' && suggestion.user_id === userId && inActiveFile(target, suggestion) && suggestion.id !== current?.draft.serverId
    );
}

function adoptSuggestion(target: ReviewTarget, source: ReviewSource, suggestion: SuggestionView): DraftOwner {
    return {
        target,
        source,
        draft: {
            anchor_start: suggestion.anchor_start,
            anchor_end: suggestion.anchor_end,
            replacement: suggestion.replacement_text,
            caret: suggestion.replacement_text.length,
            serverId: suggestion.id
        }
    };
}

function ownPendingDraftTouching(target: ReviewTarget, source: ReviewSource, edit: CapturedEdit): DraftOwner | null {
    for (const suggestion of ownPendingSuggestions(target)) {
        const range = resolveRange(source.doc, suggestion);
        if (range && touches(range, edit)) return adoptSuggestion(target, source, suggestion);
    }
    return null;
}

function draftToContinue(target: ReviewTarget, source: ReviewSource, edit: CapturedEdit) {
    const currentRange = current ? resolveDraft(current) : null;
    if (current && currentRange && touches(currentRange, edit)) return { owner: current, range: currentRange };
    const adopted = ownPendingDraftTouching(target, source, edit);
    const adoptedRange = adopted ? resolveDraft(adopted) : null;
    return adopted && adoptedRange ? { owner: adopted, range: adoptedRange } : null;
}

function makeCurrent(owner: DraftOwner) {
    if (current && current !== owner) endLiveDraft();
    current = owner;
}

function applyEdit(edit: CapturedEdit, state: EditorState): number | null {
    const context = activeContext();
    if (!context) return null;
    const { target, source } = context;

    const continued = draftToContinue(target, source, edit);
    const owner: DraftOwner = continued?.owner ?? {
        target,
        source,
        draft: { anchor_start: '', anchor_end: '', replacement: '', caret: 0, serverId: null }
    };
    makeCurrent(owner);
    const merged = continued ? mergeEdit(continued.range, edit, state.doc.length) : startDraft(edit);

    if (isEmptyDraft(merged)) {
        clearTimeout(saveTimer);
        if (owner.draft.serverId) queueWithdraw(target, owner.draft.serverId);
        current = null;
        publish();
        return merged.cursor;
    }

    Object.assign(owner.draft, anchorRange(source.text, merged.from, merged.to), { replacement: merged.replacement, caret: merged.caret });
    publish();
    scheduleSave();
    return merged.cursor;
}

function cursorAtDraftEnd(state: EditorState) {
    const range = current ? resolveDraft(current) : null;
    const selection = state.selection.main;
    return range && selection.empty && selection.head === range.to ? range : null;
}

function moveCaret(step: number, state: EditorState) {
    const range = cursorAtDraftEnd(state);
    if (!current || !range) return false;
    const caret = current.draft.caret + step;
    if (caret < 0 || caret > range.replacement.length) return false;
    current.draft.caret = caret;
    publish();
    return true;
}

function deleteInside(direction: 'backward' | 'forward', state: EditorState) {
    const range = cursorAtDraftEnd(state);
    if (!current || !range) return false;
    const caret = current.draft.caret;
    const canDelete = direction === 'backward' ? caret > 0 : caret < range.replacement.length;
    if (!canDelete) return false;
    const from = direction === 'backward' ? range.to - 1 : range.to;
    applyEdit({ from, to: from + 1, original: '', replacement: '', direction }, state);
    return true;
}

function placeCaret(suggestionId: string, offset: number): number | null {
    const context = activeContext();
    if (!context) return null;
    if (!isLiveDraftSuggestion(suggestionId)) {
        const suggestion = ownPendingSuggestions(context.target).find((candidate) => candidate.id === suggestionId);
        if (!suggestion) return null;
        makeCurrent(adoptSuggestion(context.target, context.source, suggestion));
    }
    const range = current ? resolveDraft(current) : null;
    if (!current || !range) return null;
    current.draft.caret = Math.max(0, Math.min(offset, range.replacement.length));
    publish();
    return range.to;
}

function selectionMoved() {
    if (!current || current.draft.caret === current.draft.replacement.length) return;
    current.draft.caret = current.draft.replacement.length;
    publish();
}

export const suggestingHandlers: SuggestingHandlers = {
    onEdit: applyEdit,
    onMoveCaret: moveCaret,
    onDeleteInside: deleteInside,
    onPlaceCaret: placeCaret,
    onSelectionMoved: selectionMoved
};
