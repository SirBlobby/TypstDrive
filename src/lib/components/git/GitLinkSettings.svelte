<script lang="ts">
    import Icon from '@iconify/svelte';
    import ConfirmModal from '../ConfirmModal.svelte';
    import { deleteGitLink, parseExcludedPaths, updateGitLink, type GitLink, type GitLinkUpdate } from '#lib/ts/git';

    let { projectId, link, onupdated, onunlinked }: {
        projectId: string;
        link: GitLink;
        onupdated: (link: GitLink) => void;
        onunlinked: () => void;
    } = $props();

    const fieldClass = 'rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none';

    let token = $state('');
    let username = $state(link.username ?? '');
    let excludedText = $state(link.excluded_paths.join('\n'));
    let busy = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);
    let confirmUnlink = $state(false);

    async function save(update: GitLinkUpdate) {
        busy = true;
        message = null;
        try {
            onupdated(await updateGitLink(projectId, update));
            token = '';
            message = { kind: 'success', text: 'Saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = false;
    }

    function saveSettings() {
        const update: GitLinkUpdate = { username, excluded_paths: parseExcludedPaths(excludedText) };
        if (token.trim()) update.token = token;
        save(update);
    }

    async function unlink() {
        confirmUnlink = false;
        busy = true;
        message = null;
        try {
            await deleteGitLink(projectId);
            onunlinked();
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = false;
    }
</script>

<div class="flex flex-col gap-3">
    <label class="flex flex-col gap-1.5 text-xs">
        <span class="font-medium text-[var(--color-ink-muted)]">Access token</span>
        <input bind:value={token} type="password" autocomplete="off" placeholder={link.has_token ? 'Saved. Enter a new token to replace it.' : 'No token saved'} class={fieldClass} />
    </label>
    {#if link.provider === 'generic'}
        <label class="flex flex-col gap-1.5 text-xs">
            <span class="font-medium text-[var(--color-ink-muted)]">Username</span>
            <input bind:value={username} autocomplete="off" placeholder="git" class={fieldClass} />
        </label>
    {/if}
    <label class="flex flex-col gap-1.5 text-xs">
        <span class="font-medium text-[var(--color-ink-muted)]">Exclude paths</span>
        <textarea bind:value={excludedText} rows="3" placeholder={'data/\n*.csv'} class="{fieldClass} font-mono"></textarea>
    </label>

    {#if message}
        <p class="text-xs {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
    {/if}

    <div class="flex items-center justify-between gap-2">
        <div class="flex gap-2">
            <button onclick={() => (confirmUnlink = true)} disabled={busy} class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 disabled:opacity-60">
                <Icon icon="mdi:link-variant-off" class="text-base" /> Unlink
            </button>
            {#if link.has_token}
                <button onclick={() => save({ token: '' })} disabled={busy} class="rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-60">Remove token</button>
            {/if}
        </div>
        <button onclick={saveSettings} disabled={busy} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
            <Icon icon={busy ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {busy ? 'animate-spin' : ''}" /> Save
        </button>
    </div>
</div>

{#if confirmUnlink}
    <ConfirmModal
        title="Unlink repository"
        message="Stop syncing this project with Git? The project files stay as they are, and nothing changes in the repository."
        confirmLabel="Unlink"
        onconfirm={unlink}
        onclose={() => (confirmUnlink = false)}
    />
{/if}
