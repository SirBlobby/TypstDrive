import { keymap } from '@codemirror/view';
import type { Extension } from '@codemirror/state';
import { commentsSidebarOpen } from '$lib/ts/store';
import { activeThreadStore, startCommentOnSelection } from '$lib/ts/comments';
import { activeSuggestionStore } from '$lib/ts/suggestions';
import { commentMarks } from '$lib/ts/editor-comment-marks';
import { suggestionMarks } from '$lib/ts/editor-suggestions';

function openThread(commentId: string) {
    activeThreadStore.set(commentId);
    commentsSidebarOpen.set(true);
}

function openSuggestion(suggestionId: string) {
    activeSuggestionStore.set(suggestionId);
    commentsSidebarOpen.set(true);
}

export function reviewExtensions(): Extension[] {
    return [
        commentMarks(openThread),
        suggestionMarks(openSuggestion),
        keymap.of([
            {
                key: 'Mod-Alt-m',
                preventDefault: true,
                run: () => {
                    startCommentOnSelection();
                    return true;
                }
            }
        ])
    ];
}
