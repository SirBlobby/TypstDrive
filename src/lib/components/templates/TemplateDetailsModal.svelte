<script lang="ts">
    import Icon from '@iconify/svelte';
    import { untrack } from 'svelte';
    import Modal from '../Modal.svelte';
    import TemplateThumbnail from './TemplateThumbnail.svelte';
    import TemplateAuthorsEditor from './TemplateAuthorsEditor.svelte';
    import { focusOnMount } from '#lib/ts/focus-on-mount';
    import { categoryLabel, readableName, universeUrl, type UniverseTemplate } from '#lib/ts/templates';
    import { loadSavedAuthors, saveAuthors, type TemplateAuthor } from '#lib/ts/orcid';

    let { template, oncreate, onclose }: {
        template: UniverseTemplate;
        oncreate: (title: string, authors: TemplateAuthor[]) => Promise<void>;
        onclose: () => void;
    } = $props();

    let title = $state(untrack(() => readableName(template.name)));
    let authors = $state<TemplateAuthor[]>(loadSavedAuthors());
    let creating = $state(false);
    let problem = $state('');

    async function create(event: SubmitEvent) {
        event.preventDefault();
        if (!title.trim() || creating) return;
        creating = true;
        problem = '';
        const namedAuthors = authors.filter((author) => author.name.trim());
        saveAuthors(namedAuthors);
        try {
            await oncreate(title.trim(), namedAuthors);
        } catch (err) {
            problem = (err as Error).message;
            creating = false;
        }
    }
</script>

<Modal title={readableName(template.name)} icon="mdi:view-grid-plus-outline" width="max-w-2xl" {onclose}>
    <div class="grid gap-5 sm:grid-cols-[13rem_1fr]">
        <TemplateThumbnail {template} class="aspect-[3/4] w-full rounded-md border border-[var(--color-line)]" />

        <div class="flex min-w-0 flex-col gap-3">
            <p class="text-sm text-[var(--color-ink)]">{template.description}</p>

            <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-xs">
                <dt class="text-[var(--color-ink-muted)]">Package</dt>
                <dd class="font-mono text-[var(--color-ink)]">@preview/{template.name}:{template.version}</dd>
                {#if template.authors.length > 0}
                    <dt class="text-[var(--color-ink-muted)]">By</dt>
                    <dd class="text-[var(--color-ink)]">{template.authors.join(', ')}</dd>
                {/if}
                {#if template.categories.length > 0}
                    <dt class="text-[var(--color-ink-muted)]">Type</dt>
                    <dd class="text-[var(--color-ink)]">{template.categories.map(categoryLabel).join(', ')}</dd>
                {/if}
            </dl>

            <a href={universeUrl(template)} target="_blank" rel="noopener noreferrer" class="flex items-center gap-1 self-start text-xs text-[var(--color-accent)] hover:underline">
                View on Typst Universe <Icon icon="mdi:open-in-new" class="text-sm" />
            </a>

            <form id="template-create-form" onsubmit={create} class="mt-auto flex flex-col gap-1.5 pt-2 text-xs">
                <label for="template-title" class="font-medium text-[var(--color-ink-muted)]">Name</label>
                <input
                    id="template-title"
                    use:focusOnMount
                    bind:value={title}
                    maxlength="200"
                    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                />
                <div class="mt-2">
                    <TemplateAuthorsEditor bind:authors />
                </div>
            </form>
        </div>
    </div>

    {#snippet footer()}
        {#if problem}
            <p class="mr-auto text-xs text-[var(--color-danger)]">{problem}</p>
        {/if}
        <button class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]" onclick={onclose}>
            Cancel
        </button>
        <button
            type="submit"
            form="template-create-form"
            disabled={!title.trim() || creating}
            class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
        >
            {#if creating}<Icon icon="mdi:loading" class="animate-spin text-sm" />{/if}
            {creating ? 'Creating...' : 'Create'}
        </button>
    {/snippet}
</Modal>
