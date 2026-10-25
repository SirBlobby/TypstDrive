<script lang="ts">
    import { onMount, untrack } from 'svelte';
    import Icon from '@iconify/svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import CommentThreadCard from './CommentThreadCard.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import SuggestionsList from '#lib/components/suggestions/SuggestionsList.svelte';
    import { activeSuggestionStore, loadSuggestions, suggestionsStore } from '#lib/ts/suggestions';
    import { doc } from '#lib/ts/yjs-setup';
    import { editorViewStore } from '#lib/ts/store';
    import { currentKeybind } from '#lib/ts/keybinds/keybinds-store';
    import { shortcutText } from '#lib/ts/keybinds/key-format';
    import { resolveRange } from '#lib/ts/document-anchors';
    import {
        activeThreadStore,
        buildThreads,
        commentsStore,
        createComment,
        deleteComment,
        loadComments,
        pendingCommentStore,
        type CommentThread,
        type CommentView
    } from '#lib/ts/comments';

    type PanelTab = 'open' | 'resolved' | 'suggestions';

    let { docId, canComment, canReview, isOwner, onClose }: {
        docId: string;
        canComment: boolean;
        canReview: boolean;
        isOwner: boolean;
        onClose: () => void;
    } = $props();

    const REFRESH_INTERVAL_MS = 15000;

    let filter = $state<PanelTab>('open');
    let loading = $state(true);
    let loadError = $state('');
    let commentToDelete = $state<CommentView | null>(null);

    let threads = $derived(buildThreads($commentsStore));
    let openCount = $derived(threads.filter((thread) => !thread.root.resolved).length);
    let pendingSuggestionCount = $derived($suggestionsStore.filter((suggestion) => suggestion.status === 'pending').length);
    let tabs = $derived<{ value: PanelTab; label: string; count: number }[]>([
        { value: 'open', label: 'Open', count: openCount },
        { value: 'resolved', label: 'Resolved', count: 0 },
        { value: 'suggestions', label: 'Suggestions', count: pendingSuggestionCount }
    ]);
    let visibleThreads = $derived(threads.filter((thread) => (filter === 'open' ? !thread.root.resolved : thread.root.resolved)));

    async function refresh() {
        try {
            await Promise.all([loadComments(docId), loadSuggestions(docId)]);
            loadError = '';
        } catch (err) {
            loadError = (err as Error).message;
        }
        loading = false;
    }

    function jumpToThread(thread: CommentThread) {
        const view = $editorViewStore;
        if (!view || !doc) return;
        const range = resolveRange(doc, thread.root);
        if (!range) return;
        view.dispatch({ selection: { anchor: range.from, head: range.to }, scrollIntoView: true });
    }

    function toggleThread(thread: CommentThread) {
        if ($activeThreadStore === thread.root.id) {
            activeThreadStore.set(null);
            return;
        }
        activeThreadStore.set(thread.root.id);
        jumpToThread(thread);
    }

    async function startThread(content: string) {
        const created = await createComment(docId, content, { pending: $pendingCommentStore });
        pendingCommentStore.set(null);
        filter = 'open';
        activeThreadStore.set(created.id);
    }

    async function confirmDelete() {
        const comment = commentToDelete;
        commentToDelete = null;
        if (!comment) return;
        try {
            await deleteComment(docId, comment.id);
        } catch (err) {
            alert((err as Error).message);
        }
    }

    $effect(() => {
        const activeId = $activeThreadStore;
        if (!activeId) return;
        untrack(() => {
            const activeThread = threads.find((thread) => thread.root.id === activeId);
            if (activeThread) filter = activeThread.root.resolved ? 'resolved' : 'open';
            activeSuggestionStore.set(null);
        });
    });

    $effect(() => {
        if (!$activeSuggestionStore) return;
        untrack(() => {
            filter = 'suggestions';
            activeThreadStore.set(null);
        });
    });

    onMount(() => {
        refresh();
        const timer = setInterval(refresh, REFRESH_INTERVAL_MS);
        return () => clearInterval(timer);
    });
</script>

<div class="fixed right-0 top-0 bottom-0 z-[70] flex w-[22rem] max-w-full flex-col border-l border-[var(--theme-border)] bg-[var(--theme-bg)] text-[var(--theme-text)] shadow-2xl">
    <div class="flex items-center justify-between border-b border-[var(--theme-border)] px-4 py-3">
        <div class="flex items-center gap-2">
            <Icon icon="mdi:comment-text-multiple-outline" class="text-lg" />
            <h2 class="text-sm font-semibold">Comments and suggestions</h2>
        </div>
        <button onclick={onClose} class="rounded-md p-1.5 hover:bg-[var(--color-surface-sunken)]" title="Close comments" aria-label="Close comments">
            <Icon icon="mdi:close" class="text-lg" />
        </button>
    </div>

    <div class="flex gap-1 border-b border-[var(--theme-border)] px-4 py-2">
        {#each tabs as tab (tab.value)}
            <button
                type="button"
                onclick={() => (filter = tab.value)}
                class="flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition {filter === tab.value ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
            >
                {tab.label}
                {#if tab.count > 0}
                    <span class="rounded bg-[var(--color-surface-sunken)] px-1 text-[10px] font-bold">{tab.count}</span>
                {/if}
            </button>
        {/each}
    </div>

    <div class="flex-1 space-y-3 overflow-y-auto p-4">
        {#if loading}
            <div class="flex h-full items-center justify-center"><Icon icon="mdi:loading" class="animate-spin text-2xl" /></div>
        {:else if filter === 'suggestions'}
            <SuggestionsList {docId} {canReview} />
        {:else if loadError}
            <p class="rounded-md border border-[var(--color-danger)]/20 bg-[var(--color-danger)]/10 p-3 text-center text-sm text-[var(--color-danger)]">{loadError}</p>
        {:else if visibleThreads.length === 0}
            <div class="flex h-full flex-col items-center justify-center gap-2 text-center text-[var(--color-ink-muted)]">
                <Icon icon={filter === 'open' ? 'mdi:comment-outline' : 'mdi:check-all'} class="text-4xl opacity-50" />
                <p class="text-sm">{filter === 'open' ? 'No open comments' : 'No resolved comments'}</p>
                {#if filter === 'open' && canComment}
                    <p class="max-w-[16rem] text-xs">Select text in the editor and press {shortcutText(currentKeybind('addComment')) || 'the comment shortcut'}, or right-click it, to comment on it.</p>
                {/if}
            </div>
        {:else}
            {#each visibleThreads as thread (thread.root.id)}
                <CommentThreadCard
                    {thread}
                    {docId}
                    {canComment}
                    {isOwner}
                    expanded={$activeThreadStore === thread.root.id}
                    onToggle={() => toggleThread(thread)}
                    onRequestDelete={(comment) => (commentToDelete = comment)}
                />
            {/each}
        {/if}
    </div>

    {#if canComment && filter !== 'suggestions'}
        <div class="border-t border-[var(--theme-border)] p-4">
            {#if $pendingCommentStore}
                <div class="mb-2 flex items-start gap-2 rounded-md bg-amber-500/10 px-2.5 py-2">
                    <p class="min-w-0 flex-1 truncate border-l-2 border-amber-400 pl-2 text-xs italic text-[var(--color-ink-muted)]" title={$pendingCommentStore.quoted_text}>
                        {$pendingCommentStore.quoted_text}
                    </p>
                    <button type="button" onclick={() => pendingCommentStore.set(null)} title="Comment on the whole document instead" aria-label="Remove selected text" class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
                        <Icon icon="mdi:close" class="text-sm" />
                    </button>
                </div>
            {/if}
            <CommentComposer
                placeholder={$pendingCommentStore ? 'Comment on the selected text...' : 'Comment on the document...'}
                submitLabel="Comment"
                onSubmit={startThread}
            />
        </div>
    {/if}
</div>

{#if commentToDelete}
    <ConfirmModal
        title="Delete comment"
        message={commentToDelete.parent_id ? 'This reply will be permanently deleted.' : 'This comment and all of its replies will be permanently deleted.'}
        confirmLabel="Delete"
        onconfirm={confirmDelete}
        onclose={() => (commentToDelete = null)}
    />
{/if}
