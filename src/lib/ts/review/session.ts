import { writable } from 'svelte/store';
import { activeThreadStore, commentsStore, loadComments, pendingCommentStore } from './comments';
import { activeSuggestionStore, loadSuggestions, suggestingModeStore, suggestionsStore } from './suggestions';
import { endLiveDraft } from './suggestion-drafts';
import { currentReviewTarget, reviewTargetStore, type ReviewTarget } from './target';
import { reviewChangedElsewhereStore } from './live';

const REFRESH_INTERVAL_MS = 60000;

export const reviewLoadStore = writable<{ loaded: boolean; error: string }>({ loaded: false, error: '' });

let refreshTimer: ReturnType<typeof setInterval> | undefined;
let stopListeningForChanges: (() => void) | null = null;

function clearDrafts() {
    pendingCommentStore.set(null);
    endLiveDraft();
}

export async function refreshReview() {
    const target = currentReviewTarget();
    if (!target) return;
    try {
        await Promise.all([loadComments(target), loadSuggestions(target)]);
        reviewLoadStore.set({ loaded: true, error: '' });
    } catch (err) {
        reviewLoadStore.set({ loaded: true, error: (err as Error).message });
    }
}

function refreshWhenVisible() {
    if (document.visibilityState === 'visible') refreshReview();
}

export function startReviewSession(target: ReviewTarget) {
    endReviewSession();
    reviewTargetStore.set(target);
    refreshReview();
    refreshTimer = setInterval(refreshWhenVisible, REFRESH_INTERVAL_MS);
    document.addEventListener('visibilitychange', refreshWhenVisible);
    stopListeningForChanges = reviewChangedElsewhereStore.subscribe((count) => {
        if (count > 0) refreshReview();
    });
}

export function switchReviewFile(fileId: string | null) {
    clearDrafts();
    reviewTargetStore.update((target) => (target ? { ...target, fileId } : target));
}

export function endReviewSession() {
    clearInterval(refreshTimer);
    if (typeof document !== 'undefined') document.removeEventListener('visibilitychange', refreshWhenVisible);
    stopListeningForChanges?.();
    stopListeningForChanges = null;
    clearDrafts();
    activeThreadStore.set(null);
    activeSuggestionStore.set(null);
    suggestingModeStore.set(false);
    commentsStore.set([]);
    suggestionsStore.set([]);
    reviewLoadStore.set({ loaded: false, error: '' });
    reviewTargetStore.set(null);
}
