import { get, writable } from 'svelte/store';
import { StateEffect } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import type { EditorFileKind } from '../file-kind';
import { rankCommands, type RankedCommand } from './slash-commands';
import type { SlashCommand, SlashContext } from './slash-command-types';

export type SlashTrigger = 'typed' | 'selection';

export interface SlashConfig {
    fileKind: EditorFileKind;
    context: SlashContext;
}

export interface SlashSession extends SlashConfig {
    trigger: SlashTrigger;
    from: number;
    to: number;
    query: string;
    selectedIndex: number;
}

export const closeTypedSlash = StateEffect.define<null>();

export const slashSessionStore = writable<SlashSession | null>(null);

let sessionView: EditorView | null = null;

export function visibleCommands(session: SlashSession): RankedCommand[] {
    return rankCommands(session.fileKind, session.context, session.query, session.trigger === 'selection');
}

export function showTypedSession(view: EditorView, config: SlashConfig, from: number, query: string) {
    sessionView = view;
    slashSessionStore.update((current) => {
        const sameQuery = current?.trigger === 'typed' && current.query === query;
        return { ...config, trigger: 'typed', from, to: from + 1 + query.length, query, selectedIndex: sameQuery ? current.selectedIndex : 0 };
    });
}

export function openSelectionSession(view: EditorView, config: SlashConfig) {
    const selection = view.state.selection.main;
    sessionView = view;
    slashSessionStore.set({ ...config, trigger: 'selection', from: selection.from, to: selection.to, query: '', selectedIndex: 0 });
}

export function setSelectionQuery(query: string) {
    slashSessionStore.update((session) => (session ? { ...session, query, selectedIndex: 0 } : session));
}

export function hideSession() {
    sessionView = null;
    slashSessionStore.set(null);
}

export function closeSlash(refocus = true) {
    const session = get(slashSessionStore);
    const view = sessionView;
    hideSession();
    if (!view || !session) return;
    if (session.trigger === 'typed') {
        view.dispatch({ effects: closeTypedSlash.of(null) });
    } else {
        view.dispatch({ selection: { anchor: session.from, head: session.to } });
    }
    if (refocus) view.focus();
}

export function moveSlashSelection(step: number) {
    slashSessionStore.update((session) => {
        if (!session) return session;
        const count = visibleCommands(session).length;
        if (count === 0) return session;
        return { ...session, selectedIndex: (session.selectedIndex + step + count) % count };
    });
}

function clearTypedQuery(view: EditorView, session: SlashSession) {
    view.dispatch({
        changes: { from: session.from, to: session.to, insert: '' },
        selection: { anchor: session.from },
        effects: closeTypedSlash.of(null)
    });
}

export function runSlashCommand(command: SlashCommand) {
    const session = get(slashSessionStore);
    const view = sessionView;
    hideSession();
    if (!view || !session) return;
    if (session.trigger === 'typed') {
        clearTypedQuery(view, session);
    } else {
        view.dispatch({ selection: { anchor: session.from, head: session.to } });
    }
    view.focus();
    command.run(view, session.context);
}

export function runSelectedSlashCommand() {
    const session = get(slashSessionStore);
    if (!session) return false;
    const chosen = visibleCommands(session)[session.selectedIndex];
    if (!chosen) return false;
    runSlashCommand(chosen.command);
    return true;
}
