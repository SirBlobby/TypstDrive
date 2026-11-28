import { get } from 'svelte/store';
import { Prec, StateField, type EditorState, type Extension, type Transaction } from '@codemirror/state';
import { EditorView, ViewPlugin, keymap, type ViewUpdate } from '@codemirror/view';
import {
    closeSlash,
    closeTypedSlash,
    hideSession,
    moveSlashSelection,
    openSelectionSession,
    runSelectedSlashCommand,
    showTypedSession,
    slashSessionStore,
    type SlashConfig
} from './slash-session';

const MAX_QUERY_LENGTH = 32;
const TRIGGER_CHARACTER = '/';

interface TypedSlash {
    from: number;
}

function insertedSlashPosition(transaction: Transaction): number | null {
    if (!transaction.isUserEvent('input.type')) return null;
    let position: number | null = null;
    let changeCount = 0;
    transaction.changes.iterChanges((fromA, toA, fromB, _toB, inserted) => {
        changeCount += 1;
        if (fromA === toA && inserted.toString() === TRIGGER_CHARACTER) position = fromB;
    });
    return changeCount === 1 ? position : null;
}

function startsWord(state: EditorState, position: number) {
    const line = state.doc.lineAt(position);
    if (position === line.from) return true;
    return /[\s([{]/.test(state.doc.sliceString(position - 1, position));
}

export function typedQuery(state: EditorState, slash: TypedSlash): string | null {
    const selection = state.selection.main;
    if (!selection.empty || selection.head <= slash.from) return null;
    if (state.doc.sliceString(slash.from, slash.from + 1) !== TRIGGER_CHARACTER) return null;
    const query = state.doc.sliceString(slash.from + 1, selection.head);
    if (/\s/.test(query) || query.startsWith(TRIGGER_CHARACTER) || query.length > MAX_QUERY_LENGTH) return null;
    return query;
}

const typedSlashField = StateField.define<TypedSlash | null>({
    create: () => null,
    update(value, transaction) {
        if (transaction.effects.some((effect) => effect.is(closeTypedSlash))) return null;
        if (value) {
            const moved = { from: transaction.changes.mapPos(value.from, 1) };
            return typedQuery(transaction.state, moved) === null ? null : moved;
        }
        const position = insertedSlashPosition(transaction);
        if (position === null || !startsWord(transaction.state, position)) return null;
        return { from: position };
    }
});

export function isSlashPaletteOpen(state: EditorState) {
    return !!state.field(typedSlashField, false);
}

function typedSessionSync(config: SlashConfig) {
    return ViewPlugin.fromClass(
        class {
            update(update: ViewUpdate) {
                const slash = update.state.field(typedSlashField);
                const hadSlash = update.startState.field(typedSlashField);
                if (slash) {
                    const query = typedQuery(update.state, slash);
                    if (query !== null) showTypedSession(update.view, config, slash.from, query);
                } else if (hadSlash && get(slashSessionStore)?.trigger === 'typed') {
                    hideSession();
                }
            }

            destroy() {
                if (get(slashSessionStore)) hideSession();
            }
        }
    );
}

function whileTypedOpen(run: () => boolean) {
    return (view: EditorView) => (view.state.field(typedSlashField) ? run() : false);
}

function paletteKeys(config: SlashConfig) {
    return Prec.highest(
        keymap.of([
            {
                key: TRIGGER_CHARACTER,
                run: (view) => {
                    if (view.state.selection.main.empty) return false;
                    openSelectionSession(view, config);
                    return true;
                }
            },
            { key: 'ArrowDown', run: whileTypedOpen(() => (moveSlashSelection(1), true)) },
            { key: 'ArrowUp', run: whileTypedOpen(() => (moveSlashSelection(-1), true)) },
            { key: 'Enter', run: whileTypedOpen(runSelectedSlashCommand) },
            { key: 'Tab', run: whileTypedOpen(runSelectedSlashCommand) },
            { key: 'Escape', run: whileTypedOpen(() => (closeSlash(), true)) }
        ])
    );
}

const closeOnBlur = EditorView.domEventHandlers({
    blur(_event, view) {
        if (view.state.field(typedSlashField)) closeSlash(false);
        return false;
    }
});

export function slashCommands(config: SlashConfig): Extension {
    return [typedSlashField, typedSessionSync(config), paletteKeys(config), closeOnBlur];
}
