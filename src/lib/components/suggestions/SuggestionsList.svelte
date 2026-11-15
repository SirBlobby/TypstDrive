<script lang="ts">
    import Icon from '@iconify/svelte';
    import SuggestionCard from './SuggestionCard.svelte';
    import { inActiveFile, itemFileId, itemSource, reviewTargetStore, type ReviewItem } from '#lib/ts/review-target';
    import {
        acceptSuggestion,
        activeSuggestionStore,
        rejectSuggestion,
        suggestionsStore,
        textChangedSince,
        withdrawSuggestion,
        type SuggestionView
    } from '#lib/ts/suggestions';

    let { canReview, showAllFiles, onReveal }: {
        canReview: boolean;
        showAllFiles: boolean;
        onReveal: (suggestion: SuggestionView) => void;
    } = $props();

    let showHandled = $state(false);

    let target = $derived($reviewTargetStore);

    function inScope(item: ReviewItem) {
        return !target || showAllFiles || inActiveFile(target, item);
    }

    function fileLabelFor(suggestion: SuggestionView) {
        const fileId = itemFileId(suggestion);
        return target?.scope === 'project' && showAllFiles && fileId ? target.filePath(fileId) : '';
    }

    let visibleSuggestions = $derived(
        $suggestionsStore.filter((suggestion) => inScope(suggestion) && (showHandled ? suggestion.status !== 'pending' : suggestion.status === 'pending'))
    );

    function select(suggestion: SuggestionView) {
        activeSuggestionStore.set(suggestion.id);
        onReveal(suggestion);
    }

    async function accept(suggestion: SuggestionView) {
        if (!target) throw new Error('The editor is not connected yet.');
        await acceptSuggestion(target, suggestion);
    }

    function hasChanged(suggestion: SuggestionView) {
        const source = target ? itemSource(target, suggestion) : null;
        return !!source && textChangedSince(source, suggestion);
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
            fileLabel={fileLabelFor(suggestion)}
            active={$activeSuggestionStore === suggestion.id}
            changedSince={suggestion.status === 'pending' && hasChanged(suggestion)}
            onSelect={() => select(suggestion)}
            onAccept={() => accept(suggestion)}
            onReject={() => (target ? rejectSuggestion(target, suggestion) : Promise.resolve())}
            onWithdraw={() => (target ? withdrawSuggestion(target, suggestion) : Promise.resolve())}
        />
    {:else}
        <div class="flex flex-col items-center gap-2 py-10 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:file-document-edit-outline" class="text-4xl opacity-50" />
            <p class="text-sm">{showHandled ? 'No handled suggestions yet' : 'No open suggestions'}</p>
            {#if !showHandled}
                <p class="max-w-[16rem] text-xs">Switch to Suggesting mode and type in the editor to suggest a change.</p>
            {/if}
        </div>
    {/each}
</div>
