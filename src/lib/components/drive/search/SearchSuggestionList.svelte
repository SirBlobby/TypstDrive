<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { SearchSuggestion } from '#lib/ts/search-suggestions';

    let { suggestions, highlighted, onselect }: {
        suggestions: SearchSuggestion[];
        highlighted: SearchSuggestion | undefined;
        onselect: (suggestion: SearchSuggestion) => void;
    } = $props();
</script>

<div role="listbox" class="absolute left-0 right-0 top-full mt-1 z-[100] rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-xl py-1">
    <p class="px-3 pt-1 pb-1.5 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Search options</p>
    {#each suggestions as suggestion (suggestion.label)}
        {#if suggestion.action.kind === 'hint'}
            <div class="flex items-center gap-3 px-3 py-1.5 text-sm text-[var(--color-ink-muted)]">
                <Icon icon={suggestion.icon} class="text-base shrink-0" />
                <span class="font-medium">{suggestion.label}</span>
                <span class="ml-auto text-xs truncate">{suggestion.description}</span>
            </div>
        {:else}
            <button
                role="option"
                aria-selected={suggestion === highlighted}
                onmousedown={(event) => event.preventDefault()}
                onclick={() => onselect(suggestion)}
                class="w-full flex items-center gap-3 px-3 py-1.5 text-sm text-left {suggestion === highlighted ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]'}"
            >
                <Icon icon={suggestion.icon} class="text-base shrink-0 text-[var(--color-accent)]" />
                <span class="font-semibold text-[var(--color-ink)] truncate">{suggestion.label}</span>
                <span class="ml-auto text-xs truncate">{suggestion.description}</span>
            </button>
        {/if}
    {/each}
</div>
