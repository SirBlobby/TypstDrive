import { Compartment, StateEffect, StateField, type Extension } from '@codemirror/state';
import { Decoration, EditorView, WidgetType, hoverTooltip } from '@codemirror/view';

export interface SuggestionMark {
    id: string;
    from: number;
    to: number;
    replacement: string;
    authorName: string;
    note: string | null;
    active: boolean;
    caret: number | null;
}

export const setSuggestionMarks = StateEffect.define<SuggestionMark[]>();
export const suggestingCompartment = new Compartment();

export const suggestionMarksField = StateField.define<SuggestionMark[]>({
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

function caretInside(mark: SuggestionMark) {
    return mark.caret !== null && mark.caret < mark.replacement.length;
}

class InsertedTextWidget extends WidgetType {
    constructor(readonly mark: SuggestionMark) {
        super();
    }

    eq(other: InsertedTextWidget) {
        const first = this.mark;
        const second = other.mark;
        return first.replacement === second.replacement && first.id === second.id && first.active === second.active && first.caret === second.caret;
    }

    toDOM() {
        const element = document.createElement('span');
        element.className = this.mark.active ? 'cm-suggestion-insert cm-suggestion-active' : 'cm-suggestion-insert';
        element.dataset.suggestionId = this.mark.id;
        if (!caretInside(this.mark)) {
            element.textContent = this.mark.replacement;
            return element;
        }
        const caretOffset = this.mark.caret ?? 0;
        const caret = document.createElement('span');
        caret.className = 'cm-suggestion-caret';
        element.append(this.mark.replacement.slice(0, caretOffset), caret, this.mark.replacement.slice(caretOffset));
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
            ranges.push(Decoration.widget({ widget: new InsertedTextWidget(mark), side: -1 }).range(mark.to));
        }
    }
    return Decoration.set(ranges, true);
});

const hideNativeCaret = EditorView.editorAttributes.compute([suggestionMarksField], (state): Record<string, string> =>
    state.field(suggestionMarksField).some(caretInside) ? { class: 'cm-suggestion-caret-inside' } : {}
);

function markAt(marks: SuggestionMark[], position: number) {
    return marks.find((mark) => position >= mark.from && position <= mark.to);
}

export function suggestionIdFromEvent(event: Event) {
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
        cursor: 'text',
        whiteSpace: 'pre-wrap'
    },
    '.cm-suggestion-caret': {
        display: 'inline-block',
        width: '0',
        height: '1.15em',
        verticalAlign: 'text-bottom',
        borderLeft: '1.5px solid currentColor',
        marginRight: '-1.5px',
        animation: 'cm-suggestion-blink 1.1s steps(1) infinite'
    },
    '@keyframes cm-suggestion-blink': { '50%': { visibility: 'hidden' } },
    '&.cm-suggestion-caret-inside .cm-cursor': { display: 'none' },
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
        hideNativeCaret,
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
