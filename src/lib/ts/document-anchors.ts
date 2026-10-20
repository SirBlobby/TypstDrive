import * as Y from 'yjs';

export interface AnchoredRange {
    anchor_start: string;
    anchor_end: string;
}

function toBase64(bytes: Uint8Array) {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
}

function fromBase64(encoded: string) {
    return Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
}

function encodeAnchor(text: Y.Text, index: number, assoc: number) {
    return toBase64(Y.encodeRelativePosition(Y.createRelativePositionFromTypeIndex(text, index, assoc)));
}

export function anchorRange(text: Y.Text, from: number, to: number): AnchoredRange {
    return { anchor_start: encodeAnchor(text, from, 0), anchor_end: encodeAnchor(text, to, -1) };
}

function resolveAnchor(doc: Y.Doc, encoded: string): number | null {
    try {
        const absolute = Y.createAbsolutePositionFromRelativePosition(Y.decodeRelativePosition(fromBase64(encoded)), doc);
        return absolute ? absolute.index : null;
    } catch {
        return null;
    }
}

export function resolveRange(doc: Y.Doc, range: { anchor_start: string | null; anchor_end: string | null }) {
    if (!range.anchor_start || !range.anchor_end) return null;
    const from = resolveAnchor(doc, range.anchor_start);
    const to = resolveAnchor(doc, range.anchor_end);
    if (from === null || to === null || to < from) return null;
    return { from, to };
}
