import { get, writable } from 'svelte/store';
import type { PersonSummary } from './people';
import { anchorRange, type AnchoredRange } from './document-anchors';
import { commentsSidebarOpen, editorViewStore } from './store';
import { announceReviewChange } from './live-review';
import { activeSource, createUrl, currentReviewTarget, itemUrl, listUrl, type ReviewTarget } from './review-target';

export interface CommentView {
    id: string;
    document_id?: string;
    file_id?: string;
    parent_id: string | null;
    author: PersonSummary | null;
    user_id: string;
    content: string;
    resolved: boolean;
    anchor_start: string | null;
    anchor_end: string | null;
    quoted_text: string | null;
    created_at: string;
    updated_at: string | null;
}

export interface CommentThread {
    root: CommentView;
    replies: CommentView[];
}

export interface PendingComment extends AnchoredRange {
    quoted_text: string;
}

export const commentsStore = writable<CommentView[]>([]);
export const activeThreadStore = writable<string | null>(null);
export const pendingCommentStore = writable<PendingComment | null>(null);

async function errorText(res: Response, fallback: string) {
    return (await res.text()) || fallback;
}

export async function loadComments(target: ReviewTarget) {
    const res = await fetch(listUrl(target, 'comments'));
    if (!res.ok) throw new Error(await errorText(res, 'Failed to load comments.'));
    commentsStore.set(await res.json());
}

export async function createComment(
    target: ReviewTarget,
    content: string,
    options: { parentId?: string; fileId?: string | null; pending?: PendingComment | null } = {}
): Promise<CommentView> {
    const res = await fetch(createUrl(target, 'comments', options.fileId ?? target.fileId), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            content,
            parent_id: options.parentId ?? null,
            anchor_start: options.pending?.anchor_start ?? null,
            anchor_end: options.pending?.anchor_end ?? null,
            quoted_text: options.pending?.quoted_text ?? null
        })
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to post the comment.'));
    const created: CommentView = await res.json();
    announceReviewChange();
    await loadComments(target);
    return created;
}

export async function updateComment(target: ReviewTarget, commentId: string, changes: { content?: string; resolved?: boolean }) {
    const res = await fetch(itemUrl(target, 'comments', commentId), {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(changes)
    });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to update the comment.'));
    announceReviewChange();
    await loadComments(target);
}

export async function deleteComment(target: ReviewTarget, commentId: string) {
    const res = await fetch(itemUrl(target, 'comments', commentId), { method: 'DELETE' });
    if (!res.ok) throw new Error(await errorText(res, 'Failed to delete the comment.'));
    announceReviewChange();
    await loadComments(target);
}

export function buildThreads(comments: CommentView[]): CommentThread[] {
    const threads = new Map<string, CommentThread>();
    for (const comment of comments) {
        if (!comment.parent_id) threads.set(comment.id, { root: comment, replies: [] });
    }
    for (const comment of comments) {
        if (comment.parent_id) threads.get(comment.parent_id)?.replies.push(comment);
    }
    return [...threads.values()];
}

export function startCommentOnSelection() {
    const view = get(editorViewStore);
    const target = currentReviewTarget();
    const source = target ? activeSource(target) : null;
    if (view && source) {
        const selection = view.state.selection.main;
        if (selection.to > selection.from) {
            pendingCommentStore.set({
                ...anchorRange(source.text, selection.from, selection.to),
                quoted_text: view.state.doc.sliceString(selection.from, selection.to)
            });
        }
    }
    commentsSidebarOpen.set(true);
}
