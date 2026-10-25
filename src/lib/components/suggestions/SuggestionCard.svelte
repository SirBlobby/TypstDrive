<script lang="ts">
    import Icon from '@iconify/svelte';
    import PersonAvatar from '#lib/components/people/PersonAvatar.svelte';
    import { userStore } from '#lib/ts/auth';
    import { formatTimestamp } from '#lib/ts/item-info';
    import type { SuggestionView } from '#lib/ts/suggestions';

    let { suggestion, active, canReview, changedSince, onSelect, onAccept, onReject, onWithdraw }: {
        suggestion: SuggestionView;
        active: boolean;
        canReview: boolean;
        changedSince: boolean;
        onSelect: () => void;
        onAccept: () => Promise<void>;
        onReject: () => Promise<void>;
        onWithdraw: () => Promise<void>;
    } = $props();

    let busy = $state(false);
    let errorMessage = $state('');
    let cardElement = $state<HTMLElement | null>(null);
    let isAuthor = $derived($userStore?.id === suggestion.user_id);
    let fallbackPerson = $derived({ id: suggestion.user_id, name: 'Someone', avatar_url: null });

    const statusLabels: Record<string, string> = { accepted: 'Accepted', rejected: 'Rejected', withdrawn: 'Withdrawn' };

    $effect(() => {
        if (active) cardElement?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    });

    async function run(action: () => Promise<void>) {
        busy = true;
        errorMessage = '';
        try {
            await action();
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        busy = false;
    }
</script>

<div
    bind:this={cardElement}
    role="button"
    tabindex="0"
    onclick={onSelect}
    onkeydown={(event) => { if (event.key === 'Enter') onSelect(); }}
    class="cursor-pointer rounded-xl border bg-[var(--color-surface)] p-3 shadow-sm transition {active ? 'border-[var(--color-accent)] shadow-md' : 'border-[var(--color-line)] hover:shadow-md'} {suggestion.status !== 'pending' ? 'opacity-70' : ''}"
>
    <div class="flex items-center gap-2">
        <PersonAvatar person={suggestion.author ?? fallbackPerson} sizeClass="h-7 w-7" textClass="text-xs" />
        <div class="min-w-0 flex-1">
            <p class="truncate text-xs font-semibold text-[var(--color-ink)]">{suggestion.author?.name ?? 'Someone'}</p>
            <p class="text-[10px] text-[var(--color-ink-muted)]">{formatTimestamp(suggestion.created_at)}</p>
        </div>
        {#if suggestion.status !== 'pending'}
            <span class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] font-semibold uppercase text-[var(--color-ink-muted)]">{statusLabels[suggestion.status]}</span>
        {/if}
    </div>

    <div class="mt-2 space-y-1 font-mono text-xs">
        {#if suggestion.original_text}
            <p class="max-h-24 overflow-y-auto whitespace-pre-wrap break-words rounded bg-rose-500/10 px-2 py-1 text-rose-700 line-through dark:text-rose-300">{suggestion.original_text}</p>
        {/if}
        {#if suggestion.replacement_text}
            <p class="max-h-24 overflow-y-auto whitespace-pre-wrap break-words rounded bg-emerald-500/10 px-2 py-1 text-emerald-700 dark:text-emerald-300">{suggestion.replacement_text}</p>
        {/if}
    </div>

    {#if suggestion.note}
        <p class="mt-2 whitespace-pre-wrap text-sm text-[var(--color-ink)]">{suggestion.note}</p>
    {/if}

    {#if suggestion.status === 'pending' && changedSince}
        <p class="mt-2 flex items-center gap-1 text-[11px] text-amber-600 dark:text-amber-400">
            <Icon icon="mdi:alert-outline" class="text-sm" />
            The text changed after this was suggested.
        </p>
    {/if}

    {#if errorMessage}
        <p class="mt-2 text-xs text-[var(--color-danger)]">{errorMessage}</p>
    {/if}

    {#if suggestion.status === 'pending' && (canReview || isAuthor)}
        <div class="mt-3 flex justify-end gap-2">
            {#if isAuthor && !canReview}
                <button type="button" disabled={busy} onclick={(event) => { event.stopPropagation(); run(onWithdraw); }} class="rounded-md border border-[var(--color-line)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">Withdraw</button>
            {/if}
            {#if canReview}
                <button type="button" disabled={busy} onclick={(event) => { event.stopPropagation(); run(onReject); }} class="flex items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                    <Icon icon="mdi:close" class="text-sm" />
                    Reject
                </button>
                <button type="button" disabled={busy} onclick={(event) => { event.stopPropagation(); run(onAccept); }} class="flex items-center gap-1 rounded-md bg-emerald-600 px-2.5 py-1 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-50">
                    <Icon icon="mdi:check" class="text-sm" />
                    Accept
                </button>
            {/if}
        </div>
    {/if}
</div>
