<script lang="ts">
    import { onMount, tick, untrack } from 'svelte';
    import { get } from 'svelte/store';
    import Icon from '@iconify/svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import CommentThreadCard from './CommentThreadCard.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import SuggestionsList from '#lib/components/suggestions/SuggestionsList.svelte';
    import { activeSuggestionStore, suggestionsStore } from '#lib/ts/suggestions';
    import { refreshReview, reviewLoadStore } from '#lib/ts/review-session';
    import { editorViewStore } from '#lib/ts/store';
    import { currentKeybind } from '#lib/ts/keybinds/keybinds-store';
    import { shortcutText } from '#lib/ts/keybinds/key-format';
    import { resolveRange } from '#lib/ts/document-anchors';
    import { currentReviewTarget, inActiveFile, itemFileId, itemSource, reviewTargetStore, type ReviewItem } from '#lib/ts/review-target';
    import {
        activeThreadStore,
        buildThreads,
        commentsStore,
        createComment,
        deleteComment,
        pendingCommentStore,
        type CommentThread,
        type CommentView
    } from '#lib/ts/comments';

    type PanelTab = 'open' | 'resolved' | 'suggestions';

    let { canComment, canReview, isOwner, onClose }: {
        canComment: boolean;
        canReview: boolean;
        isOwner: boolean;
        onClose: () => void;
    } = $props();

    let filter = $state<PanelTab>('open');
    let showAllFiles = $state(false);
    let loading = $derived(!$reviewLoadStore.loaded);
    let loadError = $derived($reviewLoadStore.error);
    let commentToDelete = $state<CommentView | null>(null);

    let target = $derived($reviewTargetStore);
    let isProject = $derived(target?.scope === 'project');
    let wholeTargetLabel = $derived(isProject ? 'file' : 'document');
    let canStartThread = $derived(canComment && !!target && (!isProject || !!target.fileId));

    function inScope(item: ReviewItem) {
        return !target || showAllFiles || inActiveFile(target, item);
    }

    function fileLabelFor(item: ReviewItem) {
        const fileId = itemFileId(item);
        return target && isProject && showAllFiles && fileId ? target.filePath(fileId) : '';
    }

    let threads = $derived(buildThreads($commentsStore).filter((thread) => inScope(thread.root)));
    let openCount = $derived(threads.filter((thread) => !thread.root.resolved).length);
    let pendingSuggestionCount = $derived($suggestionsStore.filter((suggestion) => suggestion.status === 'pending' && inScope(suggestion)).length);
    let tabs = $derived<{ value: PanelTab; label: string; count: number }[]>([
        { value: 'open', label: 'Open', count: openCount },
        { value: 'resolved', label: 'Resolved', count: 0 },
        { value: 'suggestions', label: 'Suggestions', count: pendingSuggestionCount }
    ]);
    let visibleThreads = $derived(threads.filter((thread) => (filter === 'open' ? !thread.root.resolved : thread.root.resolved)));


    async function revealInEditor(item: ReviewItem & { anchor_start: string | null; anchor_end: string | null }) {
        const startTarget = currentReviewTarget();
        const fileId = itemFileId(item);
        if (!startTarget) return;
        if (fileId && !inActiveFile(startTarget, item)) {
            startTarget.openFile(fileId);
            await tick();
        }
        const view = get(editorViewStore);
        const latestTarget = currentReviewTarget();
        const source = latestTarget ? itemSource(latestTarget, item) : null;
        if (!view || !source) return;
        const range = resolveRange(source.doc, item);
        if (range) view.dispatch({ selection: { anchor: range.from, head: range.to }, scrollIntoView: true });
    }

    function toggleThread(thread: CommentThread) {
        if ($activeThreadStore === thread.root.id) {
            activeThreadStore.set(null);
            return;
        }
        activeThreadStore.set(thread.root.id);
        revealInEditor(thread.root);
    }

    async function startThread(content: string) {
        if (!target) return;
        const created = await createComment(target, content, { pending: $pendingCommentStore });
        pendingCommentStore.set(null);
        filter = 'open';
        activeThreadStore.set(created.id);
    }

    async function confirmDelete() {
        const comment = commentToDelete;
        commentToDelete = null;
        if (!comment || !target) return;
        try {
            await deleteComment(target, comment.id);
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
        refreshReview();
    });
</script>

<aside class="fixed inset-y-0 right-0 z-[70] flex w-[22rem] max-w-full flex-col border-l border-[var(--theme-border)] bg-[var(--theme-bg)] text-[var(--theme-text)] shadow-2xl md:static md:z-auto md:h-full md:flex-shrink-0 md:shadow-none">
    <div class="flex items-center justify-between border-b border-[var(--theme-border)] px-4 py-3">
        <div class="flex items-center gap-2">
            <Icon icon="mdi:comment-text-multiple-outline" class="text-lg" />
            <h2 class="text-sm font-semibold">Comments and suggestions</h2>
        </div>
        <button onclick={onClose} class="rounded-md p-1.5 hover:bg-[var(--color-surface-sunken)]" title="Close comments" aria-label="Close comments">
            <Icon icon="mdi:close" class="text-lg" />
        </button>
    </div>

    {#if isProject}
        <div class="flex gap-1 border-b border-[var(--theme-border)] px-4 py-2">
            {#each [{ all: false, label: 'This file' }, { all: true, label: 'All files' }] as option (option.label)}
                <button
                    type="button"
                    onclick={() => (showAllFiles = option.all)}
                    class="rounded-md px-2.5 py-1 text-xs font-medium transition {showAllFiles === option.all ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
                >
                    {option.label}
                </button>
            {/each}
        </div>
    {/if}

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
        {#if loading || !target}
            <div class="flex h-full items-center justify-center"><Icon icon="mdi:loading" class="animate-spin text-2xl" /></div>
        {:else if filter === 'suggestions'}
            <SuggestionsList {canReview} {showAllFiles} onReveal={revealInEditor} />
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
                    {target}
                    {canComment}
                    {isOwner}
                    fileLabel={fileLabelFor(thread.root)}
                    expanded={$activeThreadStore === thread.root.id}
                    onToggle={() => toggleThread(thread)}
                    onRequestDelete={(comment) => (commentToDelete = comment)}
                />
            {/each}
        {/if}
    </div>

    {#if canStartThread && filter !== 'suggestions'}
        <div class="border-t border-[var(--theme-border)] p-4">
            {#if $pendingCommentStore}
                <div class="mb-2 flex items-start gap-2 rounded-md bg-amber-500/10 px-2.5 py-2">
                    <p class="min-w-0 flex-1 truncate border-l-2 border-amber-400 pl-2 text-xs italic text-[var(--color-ink-muted)]" title={$pendingCommentStore.quoted_text}>
                        {$pendingCommentStore.quoted_text}
                    </p>
                    <button type="button" onclick={() => pendingCommentStore.set(null)} title={`Comment on the whole ${wholeTargetLabel} instead`} aria-label="Remove selected text" class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
                        <Icon icon="mdi:close" class="text-sm" />
                    </button>
                </div>
            {/if}
            <CommentComposer
                placeholder={$pendingCommentStore ? 'Comment on the selected text...' : `Comment on the ${wholeTargetLabel}...`}
                submitLabel="Comment"
                onSubmit={startThread}
            />
        </div>
    {/if}
</aside>

{#if commentToDelete}
    <ConfirmModal
        title="Delete comment"
        message={commentToDelete.parent_id ? 'This reply will be permanently deleted.' : 'This comment and all of its replies will be permanently deleted.'}
        confirmLabel="Delete"
        onconfirm={confirmDelete}
        onclose={() => (commentToDelete = null)}
    />
{/if}
