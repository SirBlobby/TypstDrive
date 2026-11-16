import { get } from 'svelte/store';
import { resolveRange } from './document-anchors';
import { suggestingModeStore, suggestionsStore } from './suggestions';
import { liveDraftStore } from './suggestion-drafts';
import { remoteDrafts } from './live-review';
import { slashSessionStore } from './slash/slash-session';
import type { ReviewSource } from './review-target';

interface TextChange {
    from: number;
    to: number;
    insert: string;
}

function applyChanges(text: string, changes: TextChange[]) {
    const ordered = [...changes].sort((first, second) => first.from - second.from);
    let result = '';
    let position = 0;
    for (const change of ordered) {
        if (change.from < position) continue;
        result += text.slice(position, change.from) + change.insert;
        position = change.to;
    }
    return result + text.slice(position);
}

function suggestedChanges(source: ReviewSource, fileId: string | null, includeLiveDraft: boolean): TextChange[] {
    const liveDraft = includeLiveDraft ? get(liveDraftStore) : null;
    const drafts = [...remoteDrafts(source.awareness), ...(liveDraft ? [liveDraft] : [])];
    const draftServerIds = new Set(drafts.map((draft) => draft.serverId).filter(Boolean));
    const changes: TextChange[] = [];
    for (const suggestion of get(suggestionsStore)) {
        const belongsHere = (suggestion.file_id ?? null) === fileId;
        if (suggestion.status !== 'pending' || !belongsHere || draftServerIds.has(suggestion.id)) continue;
        const range = resolveRange(source.doc, suggestion);
        if (range) changes.push({ ...range, insert: suggestion.replacement_text });
    }
    for (const draft of drafts) {
        const range = resolveRange(source.doc, draft);
        if (range) changes.push({ ...range, insert: draft.replacement });
    }
    return changes;
}

function unfinishedSlashCommand(): TextChange[] {
    const session = get(slashSessionStore);
    return session?.trigger === 'typed' ? [{ from: session.from, to: session.to, insert: '' }] : [];
}

export function previewText(source: ReviewSource, fileId: string | null = null, isActiveFile = true) {
    const text = source.text.toString();
    const changes = isActiveFile ? unfinishedSlashCommand() : [];
    if (get(suggestingModeStore)) changes.push(...suggestedChanges(source, fileId, isActiveFile));
    return changes.length > 0 ? applyChanges(text, changes) : text;
}
