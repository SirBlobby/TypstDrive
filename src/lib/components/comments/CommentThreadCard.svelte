<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import CommentMessage from './CommentMessage.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import { createComment, updateComment, type CommentThread, type CommentView } from '#lib/ts/review/comments';
    import { itemFileId, type ReviewTarget } from '#lib/ts/review/target';

    let { thread, target, fileLabel = '', expanded, canComment, isOwner, onToggle, onRequestDelete }: {
        thread: CommentThread;
        target: ReviewTarget;
        fileLabel?: string;
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
            await updateComment(target, thread.root.id, { resolved: !thread.root.resolved });
        } catch (err) {
            alert((err as Error).message);
        }
        busy = false;
    }

    async function reply(content: string) {
        await createComment(target, content, { parentId: thread.root.id, fileId: itemFileId(thread.root) });
    }

    async function edit(comment: CommentView, content: string) {
        await updateComment(target, comment.id, { content });
    }
</script>

<div
    bind:this={cardElement}
    class="relative overflow-hidden rounded-xl border bg-[var(--color-surface)] transition {expanded ? 'border-[var(--color-accent)]/60 shadow-lg' : 'border-[var(--color-line)] shadow-sm hover:border-[var(--color-accent)]/30 hover:shadow-md'} {thread.root.resolved && !expanded ? 'opacity-70' : ''}"
>
    {#if expanded}
        <span class="absolute inset-y-0 left-0 w-1 bg-[var(--color-accent)]"></span>
    {/if}

    <div
        role="button"
        tabindex="0"
        onclick={onToggle}
        onkeydown={(event) => { if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) { event.preventDefault(); onToggle(); } }}
        class="cursor-pointer px-3.5 pb-3 pt-3"
    >
        {#if fileLabel}
            <p class="mb-2 inline-flex max-w-full items-center gap-1 truncate rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] font-medium text-[var(--color-ink-muted)]" title={fileLabel}>
                <Icon icon="mdi:file-document-outline" class="flex-shrink-0 text-xs" />
                <span class="truncate">{fileLabel}</span>
            </p>
        {/if}
        {#if thread.root.quoted_text}
            <div class="mb-3 flex gap-2 rounded-md bg-amber-500/10 px-2.5 py-1.5" title={thread.root.quoted_text}>
                <span class="w-0.5 flex-shrink-0 rounded-full bg-amber-400"></span>
                <p class="line-clamp-2 break-words text-xs italic text-[var(--color-ink-muted)]">{thread.root.quoted_text}</p>
            </div>
        {/if}
        <CommentMessage
            comment={thread.root}
            canDelete={isOwner}
            clampLines={!expanded}
            avatarSize="h-8 w-8"
            resolveAction={canComment ? { resolved: thread.root.resolved, busy, run: toggleResolved } : undefined}
            onEdit={(content) => edit(thread.root, content)}
            onDelete={() => onRequestDelete(thread.root)}
        />
        {#if !expanded && thread.replies.length > 0}
            <div class="mt-2.5 pl-[2.625rem]">
                <span class="inline-flex items-center gap-1 rounded-full bg-[var(--color-accent-soft)] px-2 py-0.5 text-[11px] font-medium text-[var(--color-accent)]">
                    <Icon icon="mdi:reply-outline" class="text-xs" />
                    {thread.replies.length} {thread.replies.length === 1 ? 'reply' : 'replies'}
                </span>
            </div>
        {/if}
    </div>

    {#if expanded && (thread.replies.length > 0 || canComment)}
        <div class="space-y-3 bg-[var(--color-surface-muted)] px-3.5 pb-3 pt-3">
            {#if thread.replies.length > 0}
                <div class="ml-4 space-y-3 border-l-2 border-[var(--color-line)] pl-3">
                    {#each thread.replies as replyComment (replyComment.id)}
                        <CommentMessage
                            comment={replyComment}
                            canDelete={isOwner}
                            avatarSize="h-6 w-6"
                            onEdit={(content) => edit(replyComment, content)}
                            onDelete={() => onRequestDelete(replyComment)}
                        />
                    {/each}
                </div>
            {/if}

            {#if canComment}
                <CommentComposer compact placeholder={thread.root.resolved ? 'Reply to reopen...' : 'Reply...'} submitLabel="Reply" onSubmit={reply} />
            {/if}
        </div>
    {/if}
</div>
