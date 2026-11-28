import type { CapturedEdit } from '../editor/suggesting-input';

export interface DraftRange {
    from: number;
    to: number;
    replacement: string;
    caret: number;
}

export interface MergeResult extends DraftRange {
    cursor: number;
}

export function touches(range: { from: number; to: number }, edit: CapturedEdit) {
    return edit.to >= range.from && edit.from <= range.to;
}

export function startDraft(edit: CapturedEdit): MergeResult {
    const cursor = edit.direction === 'backward' ? edit.from : edit.to;
    return { from: edit.from, to: edit.to, replacement: edit.replacement, caret: edit.replacement.length, cursor };
}

function deleteBackward(current: DraftRange, edit: CapturedEdit): MergeResult {
    const removedLength = edit.to - edit.from;
    if (current.caret > 0 && edit.to === current.to) {
        const removeFrom = Math.max(0, current.caret - removedLength);
        const replacement = current.replacement.slice(0, removeFrom) + current.replacement.slice(current.caret);
        return { ...current, replacement, caret: removeFrom, cursor: current.to };
    }
    const from = edit.from < current.from ? edit.from : Math.max(0, current.from - removedLength);
    return { ...current, from, caret: current.replacement.length, cursor: from };
}

function deleteForward(current: DraftRange, edit: CapturedEdit, documentLength: number): MergeResult {
    const removedLength = edit.to - edit.from;
    if (current.caret < current.replacement.length && edit.from === current.to) {
        const replacement = current.replacement.slice(0, current.caret) + current.replacement.slice(current.caret + removedLength);
        return { ...current, replacement, cursor: current.to };
    }
    const to = edit.to > current.to ? edit.to : Math.min(documentLength, current.to + removedLength);
    return { ...current, to, caret: current.replacement.length, cursor: to };
}

function insertAtCaret(current: DraftRange, inserted: string): MergeResult {
    const replacement = current.replacement.slice(0, current.caret) + inserted + current.replacement.slice(current.caret);
    return { ...current, replacement, caret: current.caret + inserted.length, cursor: current.to };
}

export function mergeEdit(current: DraftRange, edit: CapturedEdit, documentLength: number): MergeResult {
    if (!edit.replacement && edit.direction === 'backward') return deleteBackward(current, edit);
    if (!edit.replacement && edit.direction === 'forward') return deleteForward(current, edit, documentLength);
    if (edit.from === edit.to) return insertAtCaret(current, edit.replacement);
    const from = Math.min(current.from, edit.from);
    const to = Math.max(current.to, edit.to);
    const replacement = current.replacement + edit.replacement;
    return { from, to, replacement, caret: replacement.length, cursor: to };
}

export function isEmptyDraft(range: DraftRange) {
    return range.from === range.to && !range.replacement;
}
