import { writable } from 'svelte/store';
import type { Awareness } from 'y-protocols/awareness';
import type { AnchoredRange } from './anchors';
import type { EditingMode } from '../store';

const DRAFT_FIELD = 'suggestionDraft';
const REVIEW_VERSION_FIELD = 'reviewVersion';
const EDITING_MODE_FIELD = 'editingMode';
const REMOTE_DRAFT_PREFIX = 'remote-draft-';
const FINISHED_DRAFT_REFRESH_MS = 1500;

export interface SharedDraft extends AnchoredRange {
    replacement: string;
    serverId: string | null;
}

export interface RemoteDraft extends SharedDraft {
    id: string;
    authorName: string;
}

interface WatchedState {
    draftsSnapshot: string;
    reviewVersions: Map<number, number>;
}

export const remoteDraftsChangedStore = writable(0);
export const reviewChangedElsewhereStore = writable(0);

const watched = new Map<Awareness, WatchedState>();
let localReviewVersion = 0;
let localEditingMode: EditingMode | null = null;

export function isRemoteDraftId(suggestionId: string) {
    return suggestionId.startsWith(REMOTE_DRAFT_PREFIX);
}

export function remoteDrafts(awareness: Awareness | null | undefined): RemoteDraft[] {
    if (!awareness) return [];
    const drafts: RemoteDraft[] = [];
    awareness.getStates().forEach((state, clientId) => {
        const draft = state[DRAFT_FIELD] as SharedDraft | null | undefined;
        if (clientId === awareness.clientID || !draft) return;
        drafts.push({ ...draft, id: `${REMOTE_DRAFT_PREFIX}${clientId}`, authorName: state.user?.name ?? 'Someone' });
    });
    return drafts;
}

export function shareDraft(awareness: Awareness | null | undefined, draft: SharedDraft | null) {
    awareness?.setLocalStateField(DRAFT_FIELD, draft);
}

export function shareEditingMode(mode: EditingMode) {
    localEditingMode = mode;
    for (const awareness of watched.keys()) awareness.setLocalStateField(EDITING_MODE_FIELD, mode);
}

export function announceReviewChange() {
    localReviewVersion += 1;
    for (const awareness of watched.keys()) awareness.setLocalStateField(REVIEW_VERSION_FIELD, localReviewVersion);
}

function noticeRemoteReviewChanges(awareness: Awareness, state: WatchedState) {
    let changed = false;
    awareness.getStates().forEach((clientState, clientId) => {
        if (clientId === awareness.clientID) return;
        const version = clientState[REVIEW_VERSION_FIELD] as number | undefined;
        if (version === undefined) return;
        const previous = state.reviewVersions.get(clientId);
        if (previous !== undefined && previous !== version) changed = true;
        state.reviewVersions.set(clientId, version);
    });
    return changed;
}

function bumpReviewChanged() {
    reviewChangedElsewhereStore.update((count) => count + 1);
}

function noticeDraftChanges(awareness: Awareness, state: WatchedState) {
    const drafts = remoteDrafts(awareness);
    const snapshot = JSON.stringify(drafts);
    if (snapshot === state.draftsSnapshot) return false;
    const previousIds = (JSON.parse(state.draftsSnapshot) as RemoteDraft[]).map((draft) => draft.id);
    const finishedDraft = previousIds.some((id) => !drafts.some((draft) => draft.id === id));
    if (finishedDraft) setTimeout(bumpReviewChanged, FINISHED_DRAFT_REFRESH_MS);
    state.draftsSnapshot = snapshot;
    return true;
}

export function watchLiveReview(awareness: Awareness) {
    const state: WatchedState = { draftsSnapshot: '[]', reviewVersions: new Map() };
    watched.set(awareness, state);
    awareness.setLocalStateField(REVIEW_VERSION_FIELD, localReviewVersion);
    awareness.setLocalStateField(EDITING_MODE_FIELD, localEditingMode);

    const handleChange = () => {
        if (noticeDraftChanges(awareness, state)) remoteDraftsChangedStore.update((count) => count + 1);
        if (noticeRemoteReviewChanges(awareness, state)) bumpReviewChanged();
    };
    awareness.on('change', handleChange);

    return () => {
        awareness.off('change', handleChange);
        watched.delete(awareness);
    };
}
