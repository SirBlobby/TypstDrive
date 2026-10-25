<script lang="ts">
    import Icon from '@iconify/svelte';
    import DriveEmptyState from './DriveEmptyState.svelte';
    import DriveSectionTitle from './DriveSectionTitle.svelte';
    import { starKey, starredStore } from '#lib/ts/stars';
    import {
        MINIMUM_CONTENT_QUERY_LENGTH,
        contentSearchParams,
        contentTypeAllowed,
        highlightParts,
        nameText,
        searchDocumentText,
        tokenValue,
        type ContentSearchResult,
        type DriveSearch
    } from '#lib/ts/drive-search';

    let { search, text }: { search: DriveSearch; text: string } = $props();

    let results = $state<ContentSearchResult[]>([]);
    let loading = $state(false);
    let errorMessage = $state<string | null>(null);

    let queryTooShort = $derived(text.length < MINIMUM_CONTENT_QUERY_LENGTH);
    let requestParams = $derived(contentTypeAllowed(search) ? contentSearchParams(search, text).toString() : null);
    let starredOnly = $derived(tokenValue(search, 'is') === 'starred');
    let titleText = $derived(nameText(search));
    let visibleResults = $derived(
        results.filter((result) =>
            (!starredOnly || $starredStore.has(starKey('document', result.id))) &&
            (!titleText || result.title.toLowerCase().includes(titleText))
        )
    );

    $effect(() => {
        const params = requestParams;
        results = [];
        errorMessage = null;
        if (text.length < MINIMUM_CONTENT_QUERY_LENGTH || params === null) {
            loading = false;
            return;
        }

        loading = true;
        const controller = new AbortController();
        const timer = setTimeout(() => {
            searchDocumentText(new URLSearchParams(params), controller.signal)
                .then((found) => (results = found))
                .catch((error) => {
                    if (error.name !== 'AbortError') errorMessage = 'Search failed. Try again.';
                })
                .finally(() => {
                    if (!controller.signal.aborted) loading = false;
                });
        }, 300);

        return () => {
            clearTimeout(timer);
            controller.abort();
        };
    });

    function formatDate(value: string) {
        return new Date(value.endsWith('Z') ? value : value + 'Z').toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
    }
</script>

{#if queryTooShort}
    <DriveEmptyState icon="mdi:text-search" heading="Search inside documents" message={`Type at least ${MINIMUM_CONTENT_QUERY_LENGTH} characters after "doc:" to search the text of your documents.`} />
{:else if requestParams === null}
    <DriveEmptyState icon="mdi:text-search" heading="Only documents have text" message={'Remove the type filter or set it to document, typst or latex to search inside documents.'} />
{:else if loading}
    <div class="min-h-[45vh] flex items-center justify-center text-[var(--color-ink-muted)]">
        <Icon icon="mdi:loading" class="text-3xl animate-spin" />
    </div>
{:else if errorMessage}
    <DriveEmptyState icon="mdi:alert-circle-outline" heading="Search failed" message={errorMessage} />
{:else if visibleResults.length === 0}
    <DriveEmptyState icon="mdi:magnify-remove-outline" heading="No matches" message={`No document matching your filters contains "${text}".`} />
{:else}
    <section>
        <DriveSectionTitle title={`${visibleResults.length} document${visibleResults.length === 1 ? '' : 's'} containing "${text}"`} />
        <ul class="flex flex-col gap-2">
            {#each visibleResults as result (result.id)}
                <li>
                    <a href={`/doc/${result.id}`} class="block p-4 bg-[var(--color-surface)] border border-[var(--color-line)] rounded-lg hover:border-[var(--color-accent)] transition">
                        <div class="flex items-center gap-2 min-w-0">
                            <Icon icon="mdi:file-document" class="text-xl text-[var(--color-accent)] shrink-0" />
                            <span class="font-semibold text-[var(--color-ink)] truncate">{result.title}</span>
                            {#if result.language === 'latex'}
                                <span class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide bg-teal-500 text-white">LaTeX</span>
                            {/if}
                            {#if result.shared}
                                <span class="rounded px-1.5 py-0.5 text-[10px] font-medium bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">Shared</span>
                            {/if}
                            <span class="ml-auto shrink-0 text-xs text-[var(--color-ink-muted)]">
                                {result.match_count} match{result.match_count === 1 ? '' : 'es'} · Edited {formatDate(result.updated_at)}
                            </span>
                        </div>
                        <p class="mt-2 text-sm text-[var(--color-ink-muted)] font-mono break-words">
                            {#each highlightParts(result.snippet, text) as part}
                                {#if part.matched}
                                    <mark class="rounded px-0.5 bg-[var(--color-accent-soft)] text-[var(--color-accent)]">{part.text}</mark>
                                {:else}
                                    {part.text}
                                {/if}
                            {/each}
                        </p>
                    </a>
                </li>
            {/each}
        </ul>
    </section>
{/if}
