import { keymap } from '@codemirror/view';
import type { Extension } from '@codemirror/state';
import { commentsSidebarOpen } from '../store';
import { activeThreadStore, startCommentOnSelection } from './comments';
import { activeSuggestionStore } from './suggestions';
import { commentMarks } from '../editor/comment-marks';
import { suggestionMarks } from '../editor/suggestion-marks';
import { isLiveDraftSuggestion } from './suggestion-drafts';
import { isRemoteDraftId } from './live';
import { currentKeybind } from '../editor/keybinds/keybinds-store';

function openThread(commentId: string) {
    activeThreadStore.set(commentId);
    commentsSidebarOpen.set(true);
}

function openSuggestion(suggestionId: string) {
    if (isLiveDraftSuggestion(suggestionId) || isRemoteDraftId(suggestionId)) return;
    activeSuggestionStore.set(suggestionId);
    commentsSidebarOpen.set(true);
}

export function reviewExtensions(): Extension[] {
    const commentKey = currentKeybind('addComment');
    return [
        commentMarks(openThread),
        suggestionMarks(openSuggestion),
        keymap.of(commentKey ? [
            {
                key: commentKey,
                preventDefault: true,
                run: () => {
                    startCommentOnSelection();
                    return true;
                }
            }
        ] : [])
    ];
}
