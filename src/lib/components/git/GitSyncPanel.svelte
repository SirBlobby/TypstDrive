<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import GitChangeList from './GitChangeList.svelte';
    import GitLinkSettings from './GitLinkSettings.svelte';
    import { formatDate } from '#lib/ts/settings-ui';
    import {
        fetchGitChanges,
        providerLabels,
        pullGit,
        pushGit,
        shortCommit,
        type GitChanges,
        type GitLink,
        type GitResolution
    } from '#lib/ts/git';

    let { projectId, link, onlinkchanged, onchanged, onunlinked }: {
        projectId: string;
        link: GitLink;
        onlinkchanged: (link: GitLink) => void;
        onchanged: () => void;
        onunlinked: () => void;
    } = $props();

    let changes = $state<GitChanges | null>(null);
    let selected = $state<string[]>([]);
    let commitMessage = $state('');
    let conflicts = $state<string[]>([]);
    let busy = $state<'load' | 'pull' | 'push' | null>(null);
    let notice = $state('');
    let error = $state('');
    let showSettings = $state(false);

    async function loadChanges() {
        busy = 'load';
        error = '';
        try {
            changes = await fetchGitChanges(projectId);
            onlinkchanged(changes.link);
            selected = changes.local.map((change) => change.path);
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
    }

    async function pull(resolution: GitResolution = 'ask') {
        busy = 'pull';
        error = '';
        notice = '';
        try {
            const result = await pullGit(projectId, resolution);
            onlinkchanged(result.link);
            conflicts = result.applied ? [] : result.conflicts;
            if (result.applied) {
                const count = result.updated.length + result.removed.length;
                notice = count === 0 ? 'Already up to date.' : `Pulled ${count} ${count === 1 ? 'file' : 'files'}.`;
                if (result.too_large.length > 0) notice += ` Skipped ${result.too_large.join(', ')} (larger than 50 MB).`;
                onchanged();
            }
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
        if (conflicts.length === 0) await loadChanges();
    }

    async function push() {
        busy = 'push';
        error = '';
        notice = '';
        try {
            const result = await pushGit(projectId, commitMessage, selected);
            onlinkchanged(result.link);
            commitMessage = '';
            notice = `Pushed ${result.pushed.length} ${result.pushed.length === 1 ? 'file' : 'files'} as commit ${shortCommit(result.commit)}.`;
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
        await loadChanges();
    }

    onMount(loadChanges);
</script>

<div class="flex flex-col gap-4">
    <div class="flex items-start gap-3 rounded-lg border border-[var(--color-line)] p-3">
        <Icon icon="mdi:source-repository" class="mt-0.5 flex-shrink-0 text-lg text-[var(--color-ink-muted)]" />
        <div class="min-w-0 flex-1">
            <a href={link.repo_url} target="_blank" rel="noopener noreferrer" class="block truncate text-sm font-medium text-[var(--color-accent)] hover:underline">{link.repo_url}</a>
            <p class="truncate text-xs text-[var(--color-ink-muted)]">
                {providerLabels[link.provider]} · <span class="font-mono">{link.branch}</span>{#if link.folder} · <span class="font-mono">{link.folder}/</span>{/if}
                {#if link.base_commit}· <span class="font-mono">{shortCommit(link.base_commit)}</span>{/if}
                {#if link.last_synced_at}· Synced {formatDate(link.last_synced_at)}{/if}
                {#if link.account_username}· As {link.account_username}{/if}
            </p>
            {#if !link.has_token}
                <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">No access token is saved, so changes cannot be pushed.</p>
            {/if}
        </div>
        <button onclick={loadChanges} disabled={busy !== null} title="Check for changes" aria-label="Check for changes" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
            <Icon icon={busy === 'load' ? 'mdi:loading' : 'mdi:refresh'} class="text-base {busy === 'load' ? 'animate-spin' : ''}" />
        </button>
    </div>

    {#if conflicts.length > 0}
        <div class="flex flex-col gap-2 rounded-lg border border-amber-500/40 bg-amber-500/10 p-3 text-xs">
            <p class="font-medium text-[var(--color-ink)]">These files changed both here and in the repository:</p>
            <p class="font-mono text-[var(--color-ink-muted)]">{conflicts.join(', ')}</p>
            <div class="flex flex-wrap justify-end gap-2">
                <button onclick={() => pull('keep_mine')} disabled={busy !== null} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-60">Keep my version</button>
                <button onclick={() => pull('keep_repository')} disabled={busy !== null} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 font-semibold text-white hover:opacity-90 disabled:opacity-60">Use repository version</button>
            </div>
        </div>
    {/if}

    {#if changes}
        <section class="flex flex-col gap-2">
            <div class="flex items-center justify-between">
                <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">From the repository</h3>
                <button onclick={() => pull()} disabled={busy !== null || !changes.behind} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-60">
                    <Icon icon={busy === 'pull' ? 'mdi:loading' : 'mdi:download'} class="text-base {busy === 'pull' ? 'animate-spin' : ''}" /> Pull
                </button>
            </div>
            {#if !changes.behind}
                <p class="text-xs text-[var(--color-ink-muted)]">Up to date with {link.branch}.</p>
            {:else if changes.remote.length === 0}
                <p class="text-xs text-[var(--color-ink-muted)]">New commits are available, but they do not change files in this project.</p>
            {:else}
                <GitChangeList changes={changes.remote} conflicts={changes.conflicts} />
            {/if}
        </section>

        <section class="flex flex-col gap-2">
            <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Your changes</h3>
            {#if changes.local.length === 0}
                <p class="text-xs text-[var(--color-ink-muted)]">Nothing to commit.</p>
            {:else}
                <GitChangeList changes={changes.local} conflicts={changes.conflicts} selectable bind:selected />
                <input bind:value={commitMessage} placeholder="Describe your changes" maxlength="5000" class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                {#if changes.behind}
                    <p class="text-xs text-amber-600 dark:text-amber-400">Pull the new commits before pushing.</p>
                {/if}
                <div class="flex justify-end">
                    <button onclick={push} disabled={busy !== null || changes.behind || selected.length === 0 || !commitMessage.trim() || !link.has_token} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
                        <Icon icon={busy === 'push' ? 'mdi:loading' : 'mdi:upload'} class="text-base {busy === 'push' ? 'animate-spin' : ''}" />
                        Commit and push {selected.length} {selected.length === 1 ? 'file' : 'files'}
                    </button>
                </div>
            {/if}
        </section>
    {:else if busy === 'load'}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Checking the repository...</p>
    {/if}

    {#if notice}
        <p class="text-xs text-[var(--color-success)]">{notice}</p>
    {/if}
    {#if error}
        <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}

    {#if link.can_manage}
        <section class="flex flex-col gap-3 border-t border-[var(--color-line)] pt-3">
            <button onclick={() => (showSettings = !showSettings)} class="flex items-center gap-1 self-start text-xs font-medium text-[var(--color-accent)] hover:underline">
                <Icon icon={showSettings ? 'mdi:chevron-down' : 'mdi:chevron-right'} class="text-base" /> Link settings
            </button>
            {#if showSettings}
                <GitLinkSettings {projectId} {link} onupdated={onlinkchanged} {onunlinked} />
            {/if}
        </section>
    {/if}
</div>
