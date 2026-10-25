<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import { brandingStore } from '#lib/ts/branding';
    import { diffLines, summarize } from '#lib/ts/line-diff';
    import { fetchVersions, renameVersion, restoreVersion, versionDay, versionKindLabels, versionTime, type DocumentVersion } from '#lib/ts/versions';
    import VersionTimeline from '#lib/components/history/VersionTimeline.svelte';
    import VersionDiffView from '#lib/components/history/VersionDiffView.svelte';
    import PersonAvatar from '#lib/components/people/PersonAvatar.svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import PromptModal from '#lib/components/PromptModal.svelte';

    type CompareTarget = 'previous' | 'current';

    let docId = $derived(page.params.id ?? '');
    let documentTitle = $state('Document');
    let canEdit = $state(false);
    let versions = $state<DocumentVersion[]>([]);
    let currentText = $state('');
    let selected = $state<DocumentVersion | null>(null);
    let compareTarget = $state<CompareTarget>('previous');
    let showUnchanged = $state(false);
    let loading = $state(true);
    let errorMessage = $state('');
    let confirmRestore = $state(false);
    let renaming = $state(false);
    let busy = $state(false);

    let previousVersion = $derived.by(() => {
        if (!selected) return null;
        const index = versions.findIndex((version) => version.id === selected!.id);
        return index >= 0 ? versions[index + 1] ?? null : null;
    });

    let diff = $derived.by(() => {
        if (!selected) return [];
        if (compareTarget === 'current') return diffLines(selected.content, currentText);
        return diffLines(previousVersion?.content ?? '', selected.content);
    });

    let changeSummary = $derived(summarize(diff));

    async function loadDocument() {
        const res = await fetch(`/api/docs/${docId}`);
        if (!res.ok) return;
        const doc = await res.json();
        documentTitle = doc.title ?? documentTitle;
        canEdit = doc.effective_role === 'owner' || doc.effective_role === 'editor';
    }

    async function loadCurrentText() {
        const res = await fetch(`/api/docs/${docId}/text`);
        if (res.ok) currentText = (await res.json()).text;
    }

    async function load() {
        loading = true;
        errorMessage = '';
        try {
            await Promise.all([loadDocument(), loadCurrentText()]);
            versions = await fetchVersions(docId);
            selected = versions.find((version) => version.id === selected?.id) ?? versions[0] ?? null;
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        loading = false;
    }

    async function submitRename(label: string) {
        renaming = false;
        if (!selected) return;
        try {
            await renameVersion(docId, selected.id, label);
            await load();
        } catch (err) {
            errorMessage = (err as Error).message;
        }
    }

    async function restoreSelected() {
        confirmRestore = false;
        if (!selected) return;
        busy = true;
        try {
            await restoreVersion(docId, selected.id);
            goto(`/doc/${docId}`);
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        busy = false;
    }

    onMount(load);
</script>

<svelte:head>
    <title>Version history - {documentTitle} - {$brandingStore.site_name}</title>
</svelte:head>

<div class="flex h-screen flex-col bg-[var(--color-surface-muted)]">
    <header class="flex items-center gap-3 border-b border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-3">
        <button onclick={() => goto(`/doc/${docId}`)} class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]" title="Back to the editor" aria-label="Back to the editor">
            <Icon icon="mdi:arrow-left" class="text-xl" />
        </button>
        <div class="min-w-0">
            <p class="text-xs text-[var(--color-ink-muted)]">Version history</p>
            <h1 class="truncate text-base font-semibold text-[var(--color-ink)]">{documentTitle}</h1>
        </div>
        <div class="ml-auto flex items-center gap-2">
            {#if selected && canEdit}
                <button onclick={() => (renaming = true)} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                    <Icon icon="mdi:tag-outline" class="text-base" />
                    {selected.label ? 'Rename' : 'Name this version'}
                </button>
                <button onclick={() => (confirmRestore = true)} disabled={busy} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50">
                    <Icon icon={busy ? 'mdi:loading' : 'mdi:restore'} class="text-base {busy ? 'animate-spin' : ''}" />
                    Restore this version
                </button>
            {/if}
        </div>
    </header>

    {#if errorMessage}
        <div class="border-b border-[var(--color-danger)]/30 bg-[var(--color-danger)]/10 px-4 py-2 text-sm text-[var(--color-danger)]">{errorMessage}</div>
    {/if}

    <div class="flex min-h-0 flex-1 flex-col-reverse md:flex-row">
        <main class="min-h-0 flex-1 overflow-y-auto p-4 md:p-6">
            {#if loading}
                <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading history...</p>
            {:else if !selected}
                <div class="flex h-full flex-col items-center justify-center gap-2 text-center text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:history" class="text-5xl opacity-50" />
                    <p class="text-sm font-medium text-[var(--color-ink)]">No versions yet</p>
                    <p class="max-w-sm text-sm">A version is saved every time someone leaves the editor after making changes, or when you click Save version.</p>
                </div>
            {:else}
                <div class="mb-4 flex flex-wrap items-center gap-3">
                    <div class="flex -space-x-1.5">
                        {#each selected.editors as editor (editor.id)}
                            <PersonAvatar person={editor} sizeClass="h-8 w-8" textClass="text-xs" />
                        {/each}
                    </div>
                    <div class="min-w-0">
                        <p class="text-sm font-semibold text-[var(--color-ink)]">
                            {selected.label ?? `${versionKindLabels[selected.kind]} version`}
                        </p>
                        <p class="text-xs text-[var(--color-ink-muted)]">
                            {versionDay(selected)} at {versionTime(selected)} · Edited by {selected.editors.map((editor) => editor.name).join(', ') || 'unknown'}
                        </p>
                    </div>
                    <div class="ml-auto flex items-center gap-3">
                        <span class="text-xs font-semibold text-emerald-600 dark:text-emerald-400">+{changeSummary.added}</span>
                        <span class="text-xs font-semibold text-rose-600 dark:text-rose-400">-{changeSummary.removed}</span>
                        <select bind:value={compareTarget} aria-label="Compare with" class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-xs text-[var(--color-ink)]">
                            <option value="previous">Changes in this version</option>
                            <option value="current">Compare with current document</option>
                        </select>
                        <label class="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
                            <input type="checkbox" bind:checked={showUnchanged} class="rounded border-[var(--color-line)]" />
                            Show all lines
                        </label>
                    </div>
                </div>
                <VersionDiffView lines={diff} {showUnchanged} />
            {/if}
        </main>

        <aside class="h-72 flex-shrink-0 border-b border-[var(--color-line)] bg-[var(--color-surface)] md:h-auto md:w-80 md:border-b-0 md:border-l">
            <VersionTimeline {versions} selectedId={selected?.id ?? null} onSelect={(version) => (selected = version)} />
        </aside>
    </div>
</div>

{#if confirmRestore && selected}
    <ConfirmModal
        title="Restore this version"
        message="The document will be replaced with this version for everyone. The current content is saved as a version first, so you can undo this from history."
        confirmLabel="Restore"
        onconfirm={restoreSelected}
        onclose={() => (confirmRestore = false)}
    />
{/if}

{#if renaming && selected}
    <PromptModal
        title="Name this version"
        label="Version name"
        icon="mdi:tag-outline"
        value={selected.label ?? ''}
        confirmLabel="Save"
        onsubmit={submitRename}
        onclose={() => (renaming = false)}
    />
{/if}
