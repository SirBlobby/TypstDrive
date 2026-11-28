import { get, writable } from 'svelte/store';
import type * as Y from 'yjs';
import type { Awareness } from 'y-protocols/awareness';

export type ReviewKind = 'comments' | 'suggestions';

export interface ReviewSource {
    doc: Y.Doc;
    text: Y.Text;
    awareness?: Awareness;
}

export interface ReviewTarget {
    scope: 'document' | 'project';
    scopeId: string;
    fileId: string | null;
    sourceFor: (fileId: string | null) => ReviewSource | null;
    filePath: (fileId: string) => string;
    openFile: (fileId: string) => void;
}

export interface ReviewItem {
    file_id?: string | null;
}

export const reviewTargetStore = writable<ReviewTarget | null>(null);

export function currentReviewTarget() {
    return get(reviewTargetStore);
}

export function listUrl(target: ReviewTarget, kind: ReviewKind) {
    const base = target.scope === 'document' ? 'docs' : 'projects';
    return `/api/${base}/${target.scopeId}/${kind}`;
}

export function createUrl(target: ReviewTarget, kind: ReviewKind, fileId = target.fileId) {
    if (target.scope === 'document') return `/api/docs/${target.scopeId}/${kind}`;
    return `/api/projects/${target.scopeId}/files/${fileId}/${kind}`;
}

export function itemUrl(target: ReviewTarget, kind: ReviewKind, itemId: string) {
    const prefix = target.scope === 'document' ? '' : 'project-';
    return `/api/${prefix}${kind}/${itemId}`;
}

export function itemFileId(item: ReviewItem) {
    return item.file_id ?? null;
}

export function activeSource(target: ReviewTarget) {
    return target.sourceFor(target.fileId);
}

export function itemSource(target: ReviewTarget, item: ReviewItem) {
    return target.sourceFor(itemFileId(item));
}

export function inActiveFile(target: ReviewTarget, item: ReviewItem) {
    return target.scope === 'document' || itemFileId(item) === target.fileId;
}
