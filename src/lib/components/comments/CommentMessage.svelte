<script lang="ts">
    import Icon from '@iconify/svelte';
    import PersonAvatar from '#lib/components/people/PersonAvatar.svelte';
    import CommentComposer from './CommentComposer.svelte';
    import { userStore } from '#lib/ts/auth';
    import { formatTimestamp } from '#lib/ts/item-info';
    import type { CommentView } from '#lib/ts/comments';

    interface ResolveAction {
        resolved: boolean;
        busy: boolean;
        run: () => void;
    }

    let { comment, canDelete, clampLines = false, avatarSize = 'h-7 w-7', resolveAction = undefined, onEdit, onDelete }: {
        comment: CommentView;
        canDelete: boolean;
        clampLines?: boolean;
        avatarSize?: string;
        resolveAction?: ResolveAction;
        onEdit: (content: string) => Promise<void>;
        onDelete: () => void;
    } = $props();

    let editing = $state(false);
    let isAuthor = $derived($userStore?.id === comment.user_id);
    let fallbackPerson = $derived({ id: comment.user_id, name: 'Someone', avatar_url: null });
    let actionButton = 'rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]';

    async function saveEdit(content: string) {
        await onEdit(content);
        editing = false;
    }
</script>

<div class="group flex gap-2.5">
    <PersonAvatar person={comment.author ?? fallbackPerson} sizeClass="{avatarSize} flex-shrink-0" textClass="text-xs" />
    <div class="min-w-0 flex-1">
        <div class="flex items-start gap-2">
            <div class="min-w-0 flex-1 leading-tight">
                <p class="truncate text-[13px] font-semibold text-[var(--color-ink)]">{comment.author?.name ?? 'Someone'}</p>
                <p class="mt-0.5 text-[11px] text-[var(--color-ink-muted)]">
                    {formatTimestamp(comment.created_at)}{comment.updated_at ? ' · edited' : ''}
                </p>
            </div>
            {#if !editing}
                <div class="-mr-1 -mt-0.5 flex flex-shrink-0 items-center">
                    {#if isAuthor || canDelete}
                        <div class="flex items-center opacity-0 transition group-hover:opacity-100 focus-within:opacity-100">
                            {#if isAuthor}
                                <button type="button" onclick={(event) => { event.stopPropagation(); editing = true; }} title="Edit" aria-label="Edit comment" class={actionButton}>
                                    <Icon icon="mdi:pencil-outline" class="text-sm" />
                                </button>
                            {/if}
                            <button type="button" onclick={(event) => { event.stopPropagation(); onDelete(); }} title="Delete" aria-label="Delete comment" class="rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]">
                                <Icon icon="mdi:trash-can-outline" class="text-sm" />
                            </button>
                        </div>
                    {/if}
                    {#if resolveAction}
                        <button
                            type="button"
                            onclick={(event) => { event.stopPropagation(); resolveAction.run(); }}
                            disabled={resolveAction.busy}
                            title={resolveAction.resolved ? 'Reopen' : 'Resolve'}
                            aria-label={resolveAction.resolved ? 'Reopen thread' : 'Resolve thread'}
                            class="ml-0.5 rounded-full p-1 text-emerald-600 transition hover:bg-emerald-500/15 disabled:opacity-50 dark:text-emerald-400"
                        >
                            <Icon icon={resolveAction.busy ? 'mdi:loading' : resolveAction.resolved ? 'mdi:restore' : 'mdi:check'} class="text-base {resolveAction.busy ? 'animate-spin' : ''}" />
                        </button>
                    {/if}
                </div>
            {/if}
        </div>
        {#if editing}
            <div class="mt-2" role="presentation" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()}>
                <CommentComposer placeholder="Edit comment" submitLabel="Save" initialValue={comment.content} onSubmit={saveEdit} onCancel={() => (editing = false)} />
            </div>
        {:else}
            <p class="mt-1.5 whitespace-pre-wrap break-words text-sm leading-relaxed text-[var(--color-ink)] {clampLines ? 'line-clamp-3' : ''}">{comment.content}</p>
        {/if}
    </div>
</div>
