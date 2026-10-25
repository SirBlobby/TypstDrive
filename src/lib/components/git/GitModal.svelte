<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '../Modal.svelte';
    import GitLinkForm from './GitLinkForm.svelte';
    import GitSyncPanel from './GitSyncPanel.svelte';
    import { createGitLink, fetchGitStatus, previewGitLink, type GitLink, type GitLinkRequest, type GitLinkStatus } from '#lib/ts/git';

    let { projectId, isOwner, onchanged, onclose }: {
        projectId: string;
        isOwner: boolean;
        onchanged: () => void;
        onclose: () => void;
    } = $props();

    let status = $state<GitLinkStatus | null>(null);
    let error = $state('');

    async function load() {
        error = '';
        try {
            status = await fetchGitStatus(projectId);
        } catch (err) {
            error = (err as Error).message;
        }
    }

    async function link(request: GitLinkRequest) {
        status = await createGitLink(projectId, request);
        onchanged();
    }

    function updateLink(link: GitLink) {
        if (status) status = { ...status, link };
    }

    onMount(load);
</script>

<Modal title="Git" icon="mdi:git" width="max-w-[620px]" {onclose}>
    {#if error}
        <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {:else if !status}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
    {:else if !status.enabled}
        <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off Git sync on this server.</p>
    {:else if status.link}
        <GitSyncPanel {projectId} link={status.link} onlinkchanged={updateLink} {onchanged} onunlinked={load} />
    {:else if isOwner}
        <GitLinkForm
            genericHostsEnabled={status.generic_hosts_enabled}
            intro="Link this project to a folder in a Git repository. Only document sources, bibliographies, images and fonts are copied into the project. Everything else stays in the repository untouched, so large repositories are fine."
            submitLabel="Link and import"
            onpreview={(request) => previewGitLink(projectId, request)}
            onsubmit={link}
        />
    {:else}
        <p class="text-sm text-[var(--color-ink-muted)]">This project is not linked to a Git repository. Ask the project owner to link one.</p>
    {/if}

    {#snippet footer()}
        <button onclick={onclose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">Done</button>
    {/snippet}
</Modal>
