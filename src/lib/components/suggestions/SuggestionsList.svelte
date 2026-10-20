<script lang="ts">
    import Icon from '@iconify/svelte';
    import SuggestionCard from './SuggestionCard.svelte';
    import { doc, text } from '$lib/ts/yjs-setup';
    import { editorViewStore } from '$lib/ts/store';
    import {
        acceptSuggestion,
        activeSuggestionStore,
        currentSuggestionRange,
        rejectSuggestion,
        suggestionsStore,
        textChangedSince,
        withdrawSuggestion,
        type SuggestionView
    } from '$lib/ts/suggestions';

    let { docId, canReview }: { docId: string; canReview: boolean } = $props();

    let showHandled = $state(false);

    let visibleSuggestions = $derived(
        $suggestionsStore.filter((suggestion) => (showHandled ? suggestion.status !== 'pending' : suggestion.status === 'pending'))
    );

    function select(suggestion: SuggestionView) {
        activeSuggestionStore.set(suggestion.id);
        const view = $editorViewStore;
        if (!view || !doc) return;
        const range = currentSuggestionRange(doc, suggestion);
        if (range) view.dispatch({ selection: { anchor: range.from, head: range.to }, scrollIntoView: true });
    }

    async function accept(suggestion: SuggestionView) {
        if (!doc || !text) throw new Error('The editor is not connected yet.');
        await acceptSuggestion(docId, doc, text, suggestion);
    }

    function hasChanged(suggestion: SuggestionView) {
        return !!doc && !!text && textChangedSince(doc, text, suggestion);
    }
</script>

<div class="space-y-3">
    <label class="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
        <input type="checkbox" bind:checked={showHandled} class="rounded border-[var(--color-line)]" />
        Show accepted and rejected
    </label>

    {#each visibleSuggestions as suggestion (suggestion.id)}
        <SuggestionCard
            {suggestion}
            {canReview}
            active={$activeSuggestionStore === suggestion.id}
            changedSince={suggestion.status === 'pending' && hasChanged(suggestion)}
            onSelect={() => select(suggestion)}
            onAccept={() => accept(suggestion)}
            onReject={() => rejectSuggestion(docId, suggestion)}
            onWithdraw={() => withdrawSuggestion(docId, suggestion)}
        />
    {:else}
        <div class="flex flex-col items-center gap-2 py-10 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:file-document-edit-outline" class="text-4xl opacity-50" />
            <p class="text-sm">{showHandled ? 'No handled suggestions yet' : 'No open suggestions'}</p>
            {#if !showHandled}
                <p class="max-w-[16rem] text-xs">Switch to Suggesting mode and type in the document to suggest a change.</p>
            {/if}
        </div>
    {/each}
</div>
