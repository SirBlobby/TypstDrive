import { get } from 'svelte/store';
import { openFind, openReplace } from '../editor-insert';
import { pageActionsStore } from './page-actions';
import { toggleLineComment } from '../editor-comments';
import { startCommentOnSelection } from '../comments';
import type { EditorFileKind } from '../editor-file-kind';
import { bestMatch } from './fuzzy-match';
import { latexSlashCommands } from './latex-slash-commands';
import { slashGroupOrder, type SlashCommand, type SlashContext } from './slash-command-types';
import { typstSlashCommands } from './typst-slash-commands';

export interface RankedCommand {
    command: SlashCommand;
    matchedIndexes: number[];
}

const sharedCommands: SlashCommand[] = [
    {
        id: 'add-comment',
        label: 'Comment',
        description: 'Start a comment thread',
        icon: 'mdi:comment-plus-outline',
        group: 'Review',
        keywords: ['discuss', 'note', 'review'],
        shortcut: 'addComment',
        isAvailable: (context) => context.canComment,
        run: () => startCommentOnSelection()
    },
    {
        id: 'toggle-line-comment',
        label: 'Toggle line comment',
        description: 'Hide this line from the output',
        icon: 'mdi:comment-off-outline',
        group: 'Tools',
        keywords: ['comment out', 'disable'],
        shortcut: 'toggleComment',
        run: (view, context) => toggleLineComment(view, context.commentMarker)
    },
    { id: 'find', label: 'Find', description: 'Search this file', icon: 'mdi:magnify', group: 'Tools', keywords: ['search'], shortcut: 'find', run: openFind },
    { id: 'replace', label: 'Find and replace', description: 'Replace text in this file', icon: 'mdi:find-replace', group: 'Tools', keywords: ['search', 'substitute'], shortcut: 'replace', run: openReplace }
];

const selectionCommandIds = new Set([
    'heading-1', 'heading-2', 'heading-3', 'normal-text', 'section', 'subsection', 'subsubsection',
    'bold', 'italic', 'underline', 'strike', 'highlight', 'inline-code', 'code-block', 'verbatim', 'quote',
    'bullet-list', 'numbered-list', 'term-list', 'link', 'footnote', 'citation', 'label', 'reference',
    'inline-math', 'block-math', 'equation', 'add-comment', 'toggle-line-comment'
]);

function commandsFor(fileKind: EditorFileKind) {
    const general = [...sharedCommands, ...get(pageActionsStore)];
    if (fileKind === 'typst') return [...typstSlashCommands, ...general];
    if (fileKind === 'latex') return [...latexSlashCommands, ...general];
    return general;
}

function groupRank(command: SlashCommand) {
    return slashGroupOrder.indexOf(command.group);
}

export function rankCommands(fileKind: EditorFileKind, context: SlashContext, query: string, forSelection: boolean): RankedCommand[] {
    const available = commandsFor(fileKind).filter(
        (command) => (command.isAvailable?.(context) ?? true) && (!forSelection || selectionCommandIds.has(command.id))
    );
    if (!query) {
        return available
            .map((command, index) => ({ command, index }))
            .sort((first, second) => groupRank(first.command) - groupRank(second.command) || first.index - second.index)
            .map(({ command }) => ({ command, matchedIndexes: [] }));
    }
    const ranked: (RankedCommand & { score: number })[] = [];
    for (const command of available) {
        const match = bestMatch(query, command.label, command.keywords);
        if (match) ranked.push({ command, score: match.score, matchedIndexes: match.matchedIndexes });
    }
    return ranked.sort((first, second) => second.score - first.score);
}
