<script lang="ts">
    import Icon from '@iconify/svelte';
    import PersonAvatar from '$lib/components/people/PersonAvatar.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import { userStore } from '$lib/ts/auth';
    import { formatTimestamp } from '$lib/ts/item-info';
    import type { CommentView } from '$lib/ts/comments';

    let { comment, canDelete, clampLines = false, onEdit, onDelete }: {
        comment: CommentView;
        canDelete: boolean;
        clampLines?: boolean;
        onEdit: (content: string) => Promise<void>;
        onDelete: () => void;
    } = $props();

    let editing = $state(false);
    let isAuthor = $derived($userStore?.id === comment.user_id);
    let fallbackPerson = $derived({ id: comment.user_id, name: 'Someone', avatar_url: null });

    async function saveEdit(content: string) {
        await onEdit(content);
        editing = false;
    }
</script>

<div class="flex gap-2.5">
    <PersonAvatar person={comment.author ?? fallbackPerson} sizeClass="h-7 w-7" textClass="text-xs" />
    <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
            <p class="truncate text-xs font-semibold text-[var(--color-ink)]">{comment.author?.name ?? 'Someone'}</p>
            <p class="flex-shrink-0 text-[10px] text-[var(--color-ink-muted)]" title={formatTimestamp(comment.created_at)}>
                {formatTimestamp(comment.created_at)}{comment.updated_at ? ' (edited)' : ''}
            </p>
            {#if !editing && (isAuthor || canDelete)}
                <div class="ml-auto flex flex-shrink-0 gap-0.5">
                    {#if isAuthor}
                        <button type="button" onclick={(event) => { event.stopPropagation(); editing = true; }} title="Edit" aria-label="Edit comment" class="rounded p-0.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]">
                            <Icon icon="mdi:pencil-outline" class="text-xs" />
                        </button>
                    {/if}
                    <button type="button" onclick={(event) => { event.stopPropagation(); onDelete(); }} title="Delete" aria-label="Delete comment" class="rounded p-0.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]">
                        <Icon icon="mdi:trash-can-outline" class="text-xs" />
                    </button>
                </div>
            {/if}
        </div>
        {#if editing}
            <div class="mt-1.5">
                <CommentComposer placeholder="Edit comment" submitLabel="Save" initialValue={comment.content} onSubmit={saveEdit} onCancel={() => (editing = false)} />
            </div>
        {:else}
            <p class="mt-0.5 whitespace-pre-wrap break-words text-sm leading-relaxed text-[var(--color-ink)] {clampLines ? 'line-clamp-3' : ''}">{comment.content}</p>
        {/if}
    </div>
</div>
