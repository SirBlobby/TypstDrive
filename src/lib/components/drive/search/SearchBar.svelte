<script lang="ts">
    import Icon from '@iconify/svelte';
    import SearchChip from './SearchChip.svelte';
    import SearchSuggestionList from './SearchSuggestionList.svelte';
    import {
        emptySearch,
        hasSearch,
        isValidTokenValue,
        parseTokenDraft,
        withToken,
        withoutToken,
        type DriveSearch
    } from '$lib/ts/drive-search';
    import { searchSuggestions, type SearchSuggestion } from '$lib/ts/search-suggestions';

    let { search = $bindable() }: { search: DriveSearch } = $props();

    let input = $state<HTMLInputElement | null>(null);
    let focused = $state(false);
    let highlightedIndex = $state(0);

    let suggestions = $derived(searchSuggestions(search.text));
    let selectableSuggestions = $derived(suggestions.filter((suggestion) => suggestion.action.kind !== 'hint'));
    let showSuggestions = $derived(focused && suggestions.length > 0);

    $effect(() => {
        suggestions;
        highlightedIndex = 0;
    });

    function applySuggestion(suggestion: SearchSuggestion) {
        if (suggestion.action.kind === 'draft') search = { ...search, text: suggestion.action.text };
        if (suggestion.action.kind === 'token') search = withToken(search, suggestion.action.token);
        input?.focus();
    }

    function commitDraft(allowSpaces: boolean) {
        const draft = parseTokenDraft(search.text);
        if (!draft || (draft.filter.allowsSpaces && !allowSpaces)) return false;
        if (!isValidTokenValue(draft.filter, draft.value)) return false;
        search = withToken(search, { key: draft.filter.key, value: draft.value });
        return true;
    }

    function moveHighlight(step: number) {
        const count = selectableSuggestions.length;
        if (count === 0) return;
        highlightedIndex = (highlightedIndex + step + count) % count;
    }

    function handleKeydown(event: KeyboardEvent) {
        const highlighted = selectableSuggestions[highlightedIndex];
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            moveHighlight(event.key === 'ArrowDown' ? 1 : -1);
        } else if ((event.key === 'Enter' || event.key === 'Tab') && showSuggestions && highlighted) {
            event.preventDefault();
            applySuggestion(highlighted);
        } else if (event.key === 'Enter') {
            if (commitDraft(true)) event.preventDefault();
        } else if (event.key === ' ') {
            if (commitDraft(false)) event.preventDefault();
        } else if (event.key === 'Backspace' && search.text === '' && search.tokens.length > 0) {
            event.preventDefault();
            search = withoutToken(search, search.tokens.length - 1);
        } else if (event.key === 'Escape') {
            if (showSuggestions) focused = false;
            else search = emptySearch();
        }
    }

    function clearSearch() {
        search = emptySearch();
        input?.focus();
    }
</script>

<div class="relative flex-1 max-w-xl">
    <div
        class="flex items-center gap-1.5 w-full bg-[var(--color-surface-muted)] border rounded-lg pl-3 pr-1.5 py-1.5 transition-colors cursor-text {focused ? 'border-[var(--color-accent)] bg-[var(--color-surface)]' : 'border-[var(--color-line)]'}"
        role="presentation"
        onclick={() => input?.focus()}
    >
        <Icon icon="mdi:magnify" class="text-lg shrink-0 text-[var(--color-ink-muted)]" />
        <div class="flex flex-1 min-w-0 items-center gap-1.5 overflow-x-auto">
            {#each search.tokens as token, index (token.key)}
                <SearchChip {token} onremove={() => (search = withoutToken(search, index))} />
            {/each}
            <input
                bind:this={input}
                bind:value={search.text}
                onkeydown={handleKeydown}
                onfocus={() => (focused = true)}
                onblur={() => (focused = false)}
                placeholder={search.tokens.length === 0 ? 'Search, or type doc:, type:, is:, before:, after:' : ''}
                aria-label="Search drive"
                aria-autocomplete="list"
                aria-expanded={showSuggestions}
                class="flex-1 min-w-24 bg-transparent border-none p-0 py-0.5 text-sm text-[var(--color-ink)] placeholder:text-[var(--color-ink-muted)] focus:outline-none focus:ring-0"
            />
        </div>
        {#if hasSearch(search)}
            <button
                aria-label="Clear search"
                title="Clear search"
                onmousedown={(event) => event.preventDefault()}
                onclick={(event) => { event.stopPropagation(); clearSearch(); }}
                class="shrink-0 p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]"
            >
                <Icon icon="mdi:close" class="text-base" />
            </button>
        {/if}
    </div>

    {#if showSuggestions}
        <SearchSuggestionList
            {suggestions}
            highlighted={selectableSuggestions[highlightedIndex]}
            onselect={applySuggestion}
        />
    {/if}
</div>
