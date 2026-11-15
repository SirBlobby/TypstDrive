import { Annotation, EditorState, Prec, type Extension, type Transaction } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { ySyncAnnotation } from 'y-codemirror.next';

export type DeleteDirection = 'backward' | 'forward' | null;

export interface CapturedEdit {
    from: number;
    to: number;
    original: string;
    replacement: string;
    direction: DeleteDirection;
}

export interface SuggestingHandlers {
    onEdit: (edit: CapturedEdit, state: EditorState) => number | null;
    onMoveCaret: (step: number, state: EditorState) => boolean;
    onDeleteInside: (direction: 'backward' | 'forward', state: EditorState) => boolean;
    onPlaceCaret: (suggestionId: string, offset: number, state: EditorState) => number | null;
    onSelectionMoved: () => void;
}

const keepCaret = Annotation.define<boolean>();

function isRemoteSync(transaction: Transaction) {
    return transaction.annotation(ySyncAnnotation) !== undefined;
}

function deleteDirection(transaction: Transaction): DeleteDirection {
    if (transaction.isUserEvent('delete.backward')) return 'backward';
    if (transaction.isUserEvent('delete.forward')) return 'forward';
    return null;
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
        replacement: transaction.newDoc.sliceString(newFrom, newTo),
        direction: deleteDirection(transaction)
    };
}

function textOffsetInWidget(widget: HTMLElement, event: MouseEvent) {
    const ownerDocument = widget.ownerDocument as Document & {
        caretPositionFromPoint?: (x: number, y: number) => { offsetNode: Node; offset: number } | null;
    };
    let node: Node | null = null;
    let offset = 0;
    const position = ownerDocument.caretPositionFromPoint?.(event.clientX, event.clientY);
    if (position) {
        node = position.offsetNode;
        offset = position.offset;
    } else {
        const range = ownerDocument.caretRangeFromPoint?.(event.clientX, event.clientY);
        if (range) {
            node = range.startContainer;
            offset = range.startOffset;
        }
    }
    if (!node || !widget.contains(node)) return widget.textContent?.length ?? 0;
    const measured = ownerDocument.createRange();
    measured.setStart(widget, 0);
    measured.setEnd(node, offset);
    return measured.toString().length;
}

function editFilter(handlers: SuggestingHandlers) {
    return EditorState.transactionFilter.of((transaction) => {
        if (!transaction.docChanged || isRemoteSync(transaction)) return transaction;
        const edit = captureEdit(transaction);
        if (!edit || edit.original === edit.replacement) return [];
        const cursor = handlers.onEdit(edit, transaction.startState);
        if (cursor === null) return [];
        const position = Math.max(0, Math.min(cursor, transaction.startState.doc.length));
        return [{ selection: { anchor: position }, scrollIntoView: true, annotations: keepCaret.of(true) }];
    });
}

function caretKeys(handlers: SuggestingHandlers) {
    return Prec.high(
        keymap.of([
            { key: 'ArrowLeft', run: (view) => handlers.onMoveCaret(-1, view.state) },
            { key: 'ArrowRight', run: (view) => handlers.onMoveCaret(1, view.state) },
            { key: 'Backspace', run: (view) => handlers.onDeleteInside('backward', view.state) },
            { key: 'Delete', run: (view) => handlers.onDeleteInside('forward', view.state) }
        ])
    );
}

function widgetClicks(handlers: SuggestingHandlers) {
    return EditorView.domEventHandlers({
        mousedown(event, view) {
            const widget = (event.target as HTMLElement | null)?.closest<HTMLElement>('.cm-suggestion-insert');
            const suggestionId = widget?.dataset.suggestionId;
            if (!widget || !suggestionId) return false;
            const position = handlers.onPlaceCaret(suggestionId, textOffsetInWidget(widget, event), view.state);
            if (position === null) return false;
            event.preventDefault();
            view.dispatch({ selection: { anchor: position }, annotations: keepCaret.of(true) });
            view.focus();
            return true;
        }
    });
}

function selectionWatcher(handlers: SuggestingHandlers) {
    return EditorView.updateListener.of((update) => {
        if (!update.selectionSet) return;
        if (update.transactions.some((transaction) => transaction.annotation(keepCaret))) return;
        handlers.onSelectionMoved();
    });
}

export function suggestingInput(handlers: SuggestingHandlers): Extension {
    return [editFilter(handlers), caretKeys(handlers), widgetClicks(handlers), selectionWatcher(handlers)];
}
