import { Compartment, EditorState, StateEffect, StateField, type Extension, type Transaction } from '@codemirror/state';
import { Decoration, EditorView, WidgetType, hoverTooltip } from '@codemirror/view';
import { ySyncAnnotation } from 'y-codemirror.next';

export interface SuggestionMark {
    id: string;
    from: number;
    to: number;
    replacement: string;
    authorName: string;
    note: string | null;
    active: boolean;
}

export interface CapturedEdit {
    from: number;
    to: number;
    original: string;
    replacement: string;
}

export const setSuggestionMarks = StateEffect.define<SuggestionMark[]>();
export const suggestingCompartment = new Compartment();

const suggestionMarksField = StateField.define<SuggestionMark[]>({
    create: () => [],
    update(marks, transaction) {
        for (const effect of transaction.effects) {
            if (effect.is(setSuggestionMarks)) return effect.value;
        }
        if (!transaction.docChanged) return marks;
        return marks.map((mark) => ({
            ...mark,
            from: transaction.changes.mapPos(mark.from, 1),
            to: Math.max(transaction.changes.mapPos(mark.from, 1), transaction.changes.mapPos(mark.to, -1))
        }));
    }
});

class InsertedTextWidget extends WidgetType {
    constructor(readonly inserted: string, readonly suggestionId: string, readonly active: boolean) {
        super();
    }

    eq(other: InsertedTextWidget) {
        return other.inserted === this.inserted && other.suggestionId === this.suggestionId && other.active === this.active;
    }

    toDOM() {
        const element = document.createElement('span');
        element.className = this.active ? 'cm-suggestion-insert cm-suggestion-active' : 'cm-suggestion-insert';
        element.dataset.suggestionId = this.suggestionId;
        element.textContent = this.inserted;
        return element;
    }

    ignoreEvent() {
        return false;
    }
}

const suggestionDecorations = EditorView.decorations.compute([suggestionMarksField], (state) => {
    const ranges = [];
    for (const mark of state.field(suggestionMarksField)) {
        if (mark.to > mark.from) {
            ranges.push(
                Decoration.mark({
                    class: mark.active ? 'cm-suggestion-delete cm-suggestion-active' : 'cm-suggestion-delete',
                    attributes: { 'data-suggestion-id': mark.id }
                }).range(mark.from, mark.to)
            );
        }
        if (mark.replacement) {
            ranges.push(Decoration.widget({ widget: new InsertedTextWidget(mark.replacement, mark.id, mark.active), side: 1 }).range(mark.to));
        }
    }
    return Decoration.set(ranges, true);
});

function markAt(marks: SuggestionMark[], position: number) {
    return marks.find((mark) => position >= mark.from && position <= mark.to);
}

function suggestionIdFromEvent(event: Event) {
    const target = event.target as HTMLElement | null;
    return target?.closest<HTMLElement>('[data-suggestion-id]')?.dataset.suggestionId ?? null;
}

function tooltipContent(mark: SuggestionMark) {
    const container = document.createElement('div');
    container.className = 'cm-suggestion-tooltip';
    const author = document.createElement('div');
    author.className = 'cm-suggestion-tooltip-author';
    author.textContent = `Suggested by ${mark.authorName}`;
    container.append(author);
    if (mark.note) {
        const note = document.createElement('div');
        note.className = 'cm-suggestion-tooltip-note';
        note.textContent = mark.note;
        container.append(note);
    }
    const hint = document.createElement('div');
    hint.className = 'cm-suggestion-tooltip-hint';
    hint.textContent = 'Click to review this suggestion';
    container.append(hint);
    return container;
}

const suggestionTheme = EditorView.baseTheme({
    '.cm-suggestion-delete': {
        textDecoration: 'line-through',
        textDecorationColor: 'rgba(225, 29, 72, 0.9)',
        backgroundColor: 'rgba(244, 63, 94, 0.12)',
        cursor: 'pointer'
    },
    '.cm-suggestion-insert': {
        color: 'rgb(5, 150, 105)',
        backgroundColor: 'rgba(16, 185, 129, 0.14)',
        borderBottom: '1px dashed rgba(5, 150, 105, 0.8)',
        cursor: 'pointer',
        whiteSpace: 'pre-wrap'
    },
    '.cm-suggestion-active': { outline: '1px solid rgba(99, 102, 241, 0.7)', borderRadius: '2px' },
    '.cm-suggestion-tooltip': { maxWidth: '300px', padding: '8px 10px', fontSize: '12px', lineHeight: '1.45' },
    '.cm-suggestion-tooltip-author': { fontWeight: '600' },
    '.cm-suggestion-tooltip-note': { marginTop: '4px', whiteSpace: 'pre-wrap' },
    '.cm-suggestion-tooltip-hint': { marginTop: '6px', opacity: '0.65', fontSize: '11px' }
});

export function suggestionMarks(onOpenSuggestion: (suggestionId: string) => void): Extension {
    return [
        suggestionMarksField,
        suggestionDecorations,
        suggestionTheme,
        hoverTooltip((view, position) => {
            const mark = markAt(view.state.field(suggestionMarksField), position);
            if (!mark) return null;
            return { pos: mark.from, end: mark.to, above: true, create: () => ({ dom: tooltipContent(mark) }) };
        }),
        EditorView.domEventHandlers({
            click(event) {
                const suggestionId = suggestionIdFromEvent(event);
                if (!suggestionId) return false;
                onOpenSuggestion(suggestionId);
                return false;
            }
        }),
        suggestingCompartment.of([])
    ];
}

function isRemoteSync(transaction: Transaction) {
    return transaction.annotation(ySyncAnnotation) !== undefined;
}

function captureEdit(transaction: Transaction): CapturedEdit | null {
    let from = Infinity;
    let to = -Infinity;
    transaction.changes.iterChangedRanges((fromA, toA) => {
        from = Math.min(from, fromA);
        to = Math.max(to, toA);
    });
    if (!Number.isFinite(from)) return null;
    const newFrom = transaction.changes.mapPos(from, -1);
    const newTo = transaction.changes.mapPos(to, 1);
    return {
        from,
        to,
        original: transaction.startState.doc.sliceString(from, to),
        replacement: transaction.newDoc.sliceString(newFrom, newTo)
    };
}

export function suggestingFilter(onCapture: (edit: CapturedEdit) => void): Extension {
    return EditorState.transactionFilter.of((transaction) => {
        if (!transaction.docChanged || isRemoteSync(transaction)) return transaction;
        const edit = captureEdit(transaction);
        if (edit && edit.original !== edit.replacement) {
            queueMicrotask(() => onCapture(edit));
        }
        return [];
    });
}
