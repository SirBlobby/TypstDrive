import { StateEffect, StateField, type Extension } from '@codemirror/state';
import { Decoration, EditorView, hoverTooltip } from '@codemirror/view';

export interface CommentMark {
    id: string;
    from: number;
    to: number;
    authorName: string;
    preview: string;
    replyCount: number;
    active: boolean;
}

export const setCommentMarks = StateEffect.define<CommentMark[]>();

const commentMarksField = StateField.define<CommentMark[]>({
    create: () => [],
    update(marks, transaction) {
        for (const effect of transaction.effects) {
            if (effect.is(setCommentMarks)) return effect.value;
        }
        if (!transaction.docChanged) return marks;
        return marks
            .map((mark) => ({ ...mark, from: transaction.changes.mapPos(mark.from, 1), to: transaction.changes.mapPos(mark.to, -1) }))
            .filter((mark) => mark.to > mark.from);
    }
});

const commentDecorations = EditorView.decorations.compute([commentMarksField], (state) => {
    const ranges = state.field(commentMarksField).map((mark) =>
        Decoration.mark({
            class: mark.active ? 'cm-comment-mark cm-comment-mark-active' : 'cm-comment-mark',
            attributes: { 'data-comment-id': mark.id }
        }).range(mark.from, mark.to)
    );
    return Decoration.set(ranges, true);
});

function markAt(marks: CommentMark[], position: number) {
    return marks.filter((mark) => position >= mark.from && position <= mark.to).sort((first, second) => (first.to - first.from) - (second.to - second.from))[0];
}

function tooltipContent(mark: CommentMark) {
    const container = document.createElement('div');
    container.className = 'cm-comment-tooltip';

    const author = document.createElement('div');
    author.className = 'cm-comment-tooltip-author';
    author.textContent = mark.authorName;

    const preview = document.createElement('div');
    preview.className = 'cm-comment-tooltip-body';
    preview.textContent = mark.preview.length > 160 ? `${mark.preview.slice(0, 160)}...` : mark.preview;

    const footer = document.createElement('div');
    footer.className = 'cm-comment-tooltip-footer';
    const replies = mark.replyCount === 1 ? '1 reply' : `${mark.replyCount} replies`;
    footer.textContent = `${mark.replyCount > 0 ? `${replies} · ` : ''}Click to open the thread`;

    container.append(author, preview, footer);
    return container;
}

const commentTheme = EditorView.baseTheme({
    '.cm-comment-mark': {
        backgroundColor: 'rgba(250, 204, 21, 0.22)',
        borderBottom: '2px solid rgba(234, 179, 8, 0.85)',
        cursor: 'pointer'
    },
    '.cm-comment-mark-active': { backgroundColor: 'rgba(250, 204, 21, 0.45)' },
    '.cm-comment-tooltip': { maxWidth: '320px', padding: '8px 10px', fontFamily: 'inherit', fontSize: '12px', lineHeight: '1.45' },
    '.cm-comment-tooltip-author': { fontWeight: '600', marginBottom: '2px' },
    '.cm-comment-tooltip-body': { whiteSpace: 'pre-wrap', wordBreak: 'break-word' },
    '.cm-comment-tooltip-footer': { marginTop: '6px', opacity: '0.65', fontSize: '11px' }
});

export function commentMarks(onOpenThread: (commentId: string) => void): Extension {
    return [
        commentMarksField,
        commentDecorations,
        commentTheme,
        hoverTooltip((view, position) => {
            const mark = markAt(view.state.field(commentMarksField), position);
            if (!mark) return null;
            return { pos: mark.from, end: mark.to, above: true, create: () => ({ dom: tooltipContent(mark) }) };
        }),
        EditorView.domEventHandlers({
            click(event, view) {
                const position = view.posAtCoords({ x: event.clientX, y: event.clientY });
                if (position === null || !view.state.selection.main.empty) return false;
                const mark = markAt(view.state.field(commentMarksField), position);
                if (!mark) return false;
                onOpenThread(mark.id);
                return false;
            }
        })
    ];
}
