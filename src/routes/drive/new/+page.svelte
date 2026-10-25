<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import { brandingStore } from '#lib/ts/branding';
    import Navbar from '#lib/components/drive/Navbar.svelte';
    import DriveSidebar from '#lib/components/drive/DriveSidebar.svelte';
    import TemplateCard from '#lib/components/templates/TemplateCard.svelte';
    import TemplateDetailsModal from '#lib/components/templates/TemplateDetailsModal.svelte';
    import type { TemplateAuthor } from '#lib/ts/orcid';
    import {
        fetchTemplates,
        createFromTemplate,
        createdItemUrl,
        categoryCounts,
        categoryLabel,
        matchesQuery,
        sortTemplates,
        type TemplateSort,
        type UniverseTemplate
    } from '#lib/ts/templates';

    const PAGE_SIZE = 48;

    let templates = $state<UniverseTemplate[]>([]);
    let loading = $state(true);
    let loadError = $state('');
    let query = $state('');
    let category = $state('all');
    let sort = $state<TemplateSort>('name');
    let visibleCount = $state(PAGE_SIZE);
    let selected = $state<UniverseTemplate | null>(null);

    let folderId = $derived(page.url.searchParams.get('folder'));
    let categoryOptions = $derived<[string, number][]>([['all', templates.length], ...categoryCounts(templates)]);
    let filtered = $derived(
        sortTemplates(
            templates.filter((template) => (category === 'all' || template.categories.includes(category)) && matchesQuery(template, query)),
            sort
        )
    );
    let visible = $derived(filtered.slice(0, visibleCount));

    $effect(() => {
        query;
        category;
        sort;
        visibleCount = PAGE_SIZE;
    });

    async function load() {
        loading = true;
        loadError = '';
        try {
            templates = await fetchTemplates();
        } catch (err) {
            loadError = (err as Error).message;
        }
        loading = false;
    }

    async function create(title: string, authors: TemplateAuthor[]) {
        if (!selected) return;
        const created = await createFromTemplate(selected, title, folderId, authors);
        if (created.authors_filled === false) alert('This template has no author field that could be filled in, so add the authors in the document yourself.');
        goto(createdItemUrl(created));
    }

    onMount(() => {
        if (!$userStore) {
            goto('/login');
            return;
        }
        load();
    });
</script>

<svelte:head>
    <title>New from template - {$brandingStore.site_name}</title>
</svelte:head>

<div class="flex h-screen flex-col">
    <Navbar />

    <div class="flex min-h-0 flex-1">
        <DriveSidebar activeView="new" />

        <main class="min-w-0 flex-1 overflow-y-auto px-6 py-6 lg:px-10">
            <div class="mx-auto flex max-w-7xl flex-col gap-5">
                <header>
                    <h1 class="text-2xl font-bold text-[var(--color-ink)]">New from template</h1>
                    <p class="mt-1 text-sm text-[var(--color-ink-muted)]">Start a document or project from a Typst Universe template.</p>
                </header>

                <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
                    <label class="relative flex-1">
                        <Icon icon="mdi:magnify" class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
                        <input
                            bind:value={query}
                            type="search"
                            placeholder="Search templates, such as thesis, IEEE or CV"
                            aria-label="Search templates"
                            class="w-full rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-10 pr-3 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                        />
                    </label>
                    <select
                        bind:value={sort}
                        aria-label="Sort templates"
                        class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                    >
                        <option value="name">By name</option>
                        <option value="recent">Recently updated</option>
                    </select>
                </div>

                {#if categoryOptions.length > 1}
                    <div class="flex flex-wrap gap-1.5" role="group" aria-label="Template type">
                        {#each categoryOptions as [option, count] (option)}
                            <button
                                onclick={() => (category = option)}
                                aria-pressed={category === option}
                                class="rounded-full border px-3 py-1 text-xs transition-colors {category === option
                                    ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                                    : 'border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                            >
                                {option === 'all' ? 'All' : categoryLabel(option)}
                                <span class="opacity-60">{count}</span>
                            </button>
                        {/each}
                    </div>
                {/if}

                {#if loading}
                    <p class="flex items-center gap-2 py-12 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading templates from Typst Universe...</p>
                {:else if loadError}
                    <div class="flex flex-col items-start gap-3 py-12">
                        <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
                        <button onclick={load} class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">Try again</button>
                    </div>
                {:else if filtered.length === 0}
                    <p class="py-12 text-sm text-[var(--color-ink-muted)]">No templates match your search.</p>
                {:else}
                    <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6">
                        {#each visible as template (template.name)}
                            <TemplateCard {template} onselect={() => (selected = template)} />
                        {/each}
                    </div>
                    {#if visible.length < filtered.length}
                        <button
                            onclick={() => (visibleCount += PAGE_SIZE)}
                            class="self-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-2 text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]"
                        >
                            Show more ({filtered.length - visible.length} left)
                        </button>
                    {/if}
                {/if}

                <p class="pt-4 text-xs text-[var(--color-ink-muted)]">
                    Templates come from <a href="https://typst.app/universe" target="_blank" rel="noopener noreferrer" class="text-[var(--color-accent)] hover:underline">Typst Universe</a>
                    and are made by the community. Each one is downloaded only when you use it.
                </p>
            </div>
        </main>
    </div>
</div>

{#if selected}
    <TemplateDetailsModal template={selected} oncreate={create} onclose={() => (selected = null)} />
{/if}
