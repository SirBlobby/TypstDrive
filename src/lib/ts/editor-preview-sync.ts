import { EditorView } from '@codemirror/view';

const CLICK_SETTLE_MS = 150;

// Calls onClick with the cursor offset after a mouse click places a plain cursor. Typing and arrow keys
// don't fire it, so the preview doesn't move while you write; the delay lets a drag-select settle first.
export function cursorClickSync(onClick: (offset: number) => void) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    return EditorView.updateListener.of((update) => {
        if (!update.transactions.some((tr) => tr.isUserEvent('select.pointer'))) return;
        clearTimeout(timer);
        timer = setTimeout(() => {
            const { ranges, main } = update.view.state.selection;
            if (ranges.length === 1 && main.empty) onClick(main.head);
        }, CLICK_SETTLE_MS);
    });
}

// Places the cursor at offset and focuses the editor. Like VS Code, an off-screen target is centred so its
// surroundings show; a visible one doesn't move the view.
export function revealOffset(view: EditorView, offset: number) {
    const anchor = Math.min(offset, view.state.doc.length);
    const coords = view.coordsAtPos(anchor);
    const box = view.scrollDOM.getBoundingClientRect();
    const visible = coords !== null && coords.top >= box.top && coords.bottom <= box.bottom;
    view.dispatch({ selection: { anchor }, effects: visible ? [] : EditorView.scrollIntoView(anchor, { y: 'center' }) });
    view.focus();
}
