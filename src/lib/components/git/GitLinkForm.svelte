<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { Snippet } from 'svelte';
    import GitImportPreview from './GitImportPreview.svelte';
    import {
        parseExcludedPaths,
        providerLabels,
        type GitImportPreview as ImportPreview,
        type GitLinkRequest,
        type GitProvider
    } from '#lib/ts/git';

    let {
        genericHostsEnabled,
        onpreview,
        onsubmit,
        submitLabel,
        intro,
        repoUrl = $bindable(''),
        showReplaceOption = true,
        ready = true,
        children
    }: {
        genericHostsEnabled: boolean;
        onpreview: (request: GitLinkRequest) => Promise<ImportPreview>;
        onsubmit: (request: GitLinkRequest) => Promise<void>;
        submitLabel: string;
        intro: string;
        repoUrl?: string;
        showReplaceOption?: boolean;
        ready?: boolean;
        children?: Snippet;
    } = $props();

    const fieldClass = 'rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none';

    let provider = $state<GitProvider>('auto');
    let branch = $state('main');
    let folder = $state('');
    let token = $state('');
    let username = $state('');
    let excludedText = $state('');
    let replaceProjectFiles = $state(true);
    let showAdvanced = $state(false);
    let preview = $state<ImportPreview | null>(null);
    let busy = $state<'preview' | 'link' | null>(null);
    let error = $state('');

    let providers = $derived((['auto', 'github', 'gitlab', 'generic'] as GitProvider[]).filter((option) => option !== 'generic' || genericHostsEnabled));
    let canSubmit = $derived(ready && repoUrl.trim().length > 0 && branch.trim().length > 0 && busy === null);

    function linkRequest(): GitLinkRequest {
        return {
            provider,
            repo_url: repoUrl,
            branch,
            folder,
            username,
            token,
            excluded_paths: parseExcludedPaths(excludedText),
            replace_project_files: showReplaceOption && replaceProjectFiles
        };
    }

    $effect(() => {
        linkRequest();
        preview = null;
    });

    async function checkRepository() {
        busy = 'preview';
        error = '';
        try {
            preview = await onpreview(linkRequest());
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
    }

    async function submit() {
        busy = 'link';
        error = '';
        try {
            await onsubmit(linkRequest());
        } catch (err) {
            error = (err as Error).message;
        }
        busy = null;
    }
</script>

<div class="flex flex-col gap-4">
    <p class="text-xs text-[var(--color-ink-muted)]">{intro}</p>

    {@render children?.()}

    <div class="grid gap-3 sm:grid-cols-[1fr_auto]">
        <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Repository address</span>
            <input bind:value={repoUrl} placeholder="https://github.com/owner/repository" class={fieldClass} />
        </label>
        <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Host</span>
            <select bind:value={provider} class={fieldClass}>
                {#each providers as option (option)}
                    <option value={option}>{providerLabels[option]}</option>
                {/each}
            </select>
        </label>
    </div>

    <div class="grid gap-3 sm:grid-cols-2">
        <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Branch</span>
            <input bind:value={branch} placeholder="main" class="{fieldClass} font-mono" />
        </label>
        <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Folder in the repository</span>
            <input bind:value={folder} placeholder="Whole repository" class="{fieldClass} font-mono" />
        </label>
    </div>

    <label class="flex flex-col gap-1.5 text-xs">
        <span class="font-medium text-[var(--color-ink-muted)]">Access token</span>
        <input bind:value={token} type="password" autocomplete="off" placeholder="Needed for private repositories and for pushing" class={fieldClass} />
        <span class="text-[var(--color-ink-muted)]">Use a token with read and write access to repository contents. It is stored encrypted.</span>
    </label>

    <button onclick={() => (showAdvanced = !showAdvanced)} class="flex items-center gap-1 self-start text-xs font-medium text-[var(--color-accent)] hover:underline">
        <Icon icon={showAdvanced ? 'mdi:chevron-down' : 'mdi:chevron-right'} class="text-base" /> More options
    </button>

    {#if showAdvanced}
        <div class="flex flex-col gap-3">
            <label class="flex flex-col gap-1.5 text-xs">
                <span class="font-medium text-[var(--color-ink-muted)]">Username (other Git servers only)</span>
                <input bind:value={username} autocomplete="off" placeholder="git" class={fieldClass} />
            </label>
            <label class="flex flex-col gap-1.5 text-xs">
                <span class="font-medium text-[var(--color-ink-muted)]">Exclude paths</span>
                <textarea bind:value={excludedText} rows="3" placeholder={'data/\n*.csv'} class="{fieldClass} font-mono"></textarea>
                <span class="text-[var(--color-ink-muted)]">One per line. A folder like data/ or a pattern like *.csv.</span>
            </label>
            {#if showReplaceOption}
                <label class="flex items-center gap-2 text-xs text-[var(--color-ink)]">
                    <input type="checkbox" bind:checked={replaceProjectFiles} />
                    Remove project files that are not in the repository
                </label>
            {/if}
        </div>
    {/if}

    {#if preview}
        <GitImportPreview {preview} />
    {/if}

    {#if error}
        <p class="text-xs text-[var(--color-danger)]">{error}</p>
    {/if}

    <div class="flex justify-end gap-2">
        <button onclick={checkRepository} disabled={!canSubmit} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-60">
            <Icon icon={busy === 'preview' ? 'mdi:loading' : 'mdi:magnify'} class="text-base {busy === 'preview' ? 'animate-spin' : ''}" />
            Check repository
        </button>
        <button onclick={submit} disabled={!canSubmit || !preview} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
            <Icon icon={busy === 'link' ? 'mdi:loading' : 'mdi:source-branch'} class="text-base {busy === 'link' ? 'animate-spin' : ''}" />
            {submitLabel}
        </button>
    </div>
</div>
