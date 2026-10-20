<script lang="ts">
    import Icon from '@iconify/svelte';
    import CommentMessage from './CommentMessage.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import { createComment, updateComment, type CommentThread, type CommentView } from '$lib/ts/comments';

    let { thread, docId, expanded, canComment, isOwner, onToggle, onRequestDelete }: {
        thread: CommentThread;
        docId: string;
        expanded: boolean;
        canComment: boolean;
        isOwner: boolean;
        onToggle: () => void;
        onRequestDelete: (comment: CommentView) => void;
    } = $props();

    let cardElement = $state<HTMLElement | null>(null);
    let busy = $state(false);

    $effect(() => {
        if (expanded) cardElement?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    });

    async function toggleResolved() {
        busy = true;
        try {
            await updateComment(docId, thread.root.id, { resolved: !thread.root.resolved });
        } catch (err) {
            alert((err as Error).message);
        }
        busy = false;
    }

    async function reply(content: string) {
        await createComment(docId, content, { parentId: thread.root.id });
    }

    async function edit(comment: CommentView, content: string) {
        await updateComment(docId, comment.id, { content });
    }
</script>

<div
    bind:this={cardElement}
    class="rounded-xl border bg-[var(--color-surface)] shadow-sm transition {expanded ? 'border-[var(--color-accent)] shadow-md' : 'border-[var(--color-line)] hover:shadow-md'} {thread.root.resolved ? 'opacity-70' : ''}"
>
    <div
        role="button"
        tabindex="0"
        onclick={onToggle}
        onkeydown={(event) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); onToggle(); } }}
        class="cursor-pointer p-3"
    >
        {#if thread.root.quoted_text}
            <p class="mb-2 truncate border-l-2 border-amber-400 pl-2 text-xs italic text-[var(--color-ink-muted)]" title={thread.root.quoted_text}>
                {thread.root.quoted_text}
            </p>
        {/if}
        <CommentMessage
            comment={thread.root}
            canDelete={isOwner}
            clampLines={!expanded}
            onEdit={(content) => edit(thread.root, content)}
            onDelete={() => onRequestDelete(thread.root)}
        />
        {#if !expanded && thread.replies.length > 0}
            <p class="mt-2 flex items-center gap-1 pl-9 text-xs font-medium text-[var(--color-accent)]">
                <Icon icon="mdi:reply-outline" class="text-sm" />
                {thread.replies.length} {thread.replies.length === 1 ? 'reply' : 'replies'}
            </p>
        {/if}
    </div>

    {#if expanded}
        <div class="space-y-3 border-t border-[var(--color-line)] px-3 pb-3 pt-3">
            {#each thread.replies as replyComment (replyComment.id)}
                <CommentMessage
                    comment={replyComment}
                    canDelete={isOwner}
                    onEdit={(content) => edit(replyComment, content)}
                    onDelete={() => onRequestDelete(replyComment)}
                />
            {/each}

            {#if canComment}
                <CommentComposer placeholder="Reply..." submitLabel="Reply" onSubmit={reply} />
                <div class="flex justify-end">
                    <button type="button" onclick={toggleResolved} disabled={busy} class="flex items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                        <Icon icon={thread.root.resolved ? 'mdi:refresh' : 'mdi:check'} class="text-sm" />
                        {thread.root.resolved ? 'Reopen' : 'Resolve'}
                    </button>
                </div>
            {/if}
        </div>
    {/if}
</div>
