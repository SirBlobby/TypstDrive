<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import BibliographyFilePicker from './BibliographyFilePicker.svelte';
    import { editorViewStore } from '#lib/ts/store';
    import {
        addCitation,
        citationMarkup,
        fetchBibliographyFiles,
        fetchCitationPolicy,
        formatForFile,
        lookupCitation,
        type AddedEntry,
        type CitationTargetKind,
        type LookupResult
    } from '#lib/ts/citations';

    let { kind, itemId, latex, onchanged }: {
        kind: CitationTargetKind;
        itemId: string;
        latex: boolean;
        onchanged: () => void;
    } = $props();

    let enabled = $state(true);
    let files = $state<string[]>([]);
    let fileName = $state('references.bib');
    let identifier = $state('');
    let preview = $state<LookupResult | null>(null);
    let added = $state<AddedEntry | null>(null);
    let busy = $state<'lookup' | 'add' | null>(null);
    let error = $state('');
    let copied = $state(false);

    let format = $derived(formatForFile(fileName));

    onMount(async () => {
        try {
            enabled = (await fetchCitationPolicy()).lookup_enabled;
            files = await fetchBibliographyFiles(kind, itemId);
        } catch (err) {
            error = (err as Error).message;
        }
    });

    async function lookUp(event?: SubmitEvent) {
        event?.preventDefault();
        if (!identifier.trim()) return;
        busy = 'lookup';
        error = '';
        added = null;
        try {
            preview = await lookupCitation(identifier, format);
        } catch (err) {
            preview = null;
            error = (err as Error).message;
        }
        busy = null;
    }

    async function addToFile() {
        busy = 'add';
        error = '';
        try {
            added = await addCitation(kind, itemId, identifier, fileName);
            if (!files.includes(added.file_name)) files = [...files, added.file_name].sort();
            preview = null;
            identifier = '';
            onchanged();
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
    }

    async function copyEntry() {
        if (!preview) return;
        try {
            await navigator.clipboard.writeText(preview.entry);
            copied = true;
            setTimeout(() => (copied = false), 2000);
        } catch {}
    }

    function insertCitation(key: string) {
        const view = $editorViewStore;
        if (!view) return;
        view.dispatch(view.state.replaceSelection(citationMarkup(key, latex)));
        view.focus();
    }
</script>

{#if !enabled}
    <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off DOI and arXiv lookups on this server.</p>
{:else}
    <div class="flex flex-col gap-4">
        <form onsubmit={lookUp} class="flex flex-col gap-1.5 text-xs">
            <label for="citation-identifier" class="font-medium text-[var(--color-ink-muted)]">DOI or arXiv ID</label>
            <div class="flex gap-2">
                <input
                    id="citation-identifier"
                    bind:value={identifier}
                    placeholder="10.1038/nature14539 or 1706.03762"
                    class="min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 font-mono text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                />
                <button type="submit" disabled={!identifier.trim() || busy !== null} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                    <Icon icon={busy === 'lookup' ? 'mdi:loading' : 'mdi:magnify'} class="text-base {busy === 'lookup' ? 'animate-spin' : ''}" />
                    Look up
                </button>
            </div>
        </form>

        <BibliographyFilePicker {files} bind:value={fileName} defaultName="references.bib" />

        {#if error}
            <p class="text-xs text-[var(--color-danger)]">{error}</p>
        {/if}

        {#if preview}
            <div class="flex flex-col gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3">
                <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                        <p class="text-sm font-medium text-[var(--color-ink)]">{preview.title}</p>
                        <p class="font-mono text-xs text-[var(--color-ink-muted)]">{citationMarkup(preview.key, latex)}</p>
                    </div>
                    <button onclick={copyEntry} title="Copy entry" aria-label="Copy entry" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                        <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
                    </button>
                </div>
                <pre class="max-h-56 overflow-auto rounded-md bg-[var(--color-surface)] p-2.5 font-mono text-[11px] leading-relaxed text-[var(--color-ink)]">{preview.entry}</pre>
                <div class="flex justify-end">
                    <button onclick={addToFile} disabled={busy !== null || !fileName} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
                        <Icon icon={busy === 'add' ? 'mdi:loading' : 'mdi:plus'} class="text-base {busy === 'add' ? 'animate-spin' : ''}" />
                        Add to {fileName}
                    </button>
                </div>
            </div>
        {/if}

        {#if added}
            <div class="flex items-center gap-3 rounded-lg border border-[var(--color-success)]/40 bg-[var(--color-success)]/10 p-3">
                <Icon icon="mdi:check-circle" class="text-xl flex-shrink-0 text-[var(--color-success)]" />
                <p class="min-w-0 flex-1 text-xs text-[var(--color-ink)]">
                    Added <span class="font-mono">{added.key}</span> to {added.file_name}.
                    {#if added.created_file && !latex}
                        Add <span class="font-mono">#bibliography("{added.file_name}")</span> where the list should appear.
                    {/if}
                </p>
                <button onclick={() => added && insertCitation(added.key)} class="flex-shrink-0 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">Insert citation</button>
            </div>
        {/if}
    </div>
{/if}
