<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '../Modal.svelte';
    import GitLinkForm from './GitLinkForm.svelte';
    import { fetchGitPolicy, importGitProject, previewGitImport, repositoryName, type GitLinkRequest, type GitPolicy } from '#lib/ts/git';

    let { folderId, onimported, onclose }: {
        folderId: string | null;
        onimported: (projectId: string) => void;
        onclose: () => void;
    } = $props();

    let policy = $state<GitPolicy | null>(null);
    let error = $state('');
    let repoUrl = $state('');
    let projectName = $state('');
    let nameEdited = $state(false);

    $effect(() => {
        const suggested = repositoryName(repoUrl);
        if (!nameEdited) projectName = suggested;
    });

    async function importProject(request: GitLinkRequest) {
        const project = await importGitProject(projectName.trim(), folderId, request);
        onimported(project.id);
    }

    onMount(async () => {
        try {
            policy = await fetchGitPolicy();
        } catch (err) {
            error = (err as Error).message;
        }
    });
</script>

<Modal title="Import from Git" icon="mdi:git" width="max-w-[620px]" {onclose}>
    {#if error}
        <p class="text-sm text-[var(--color-danger)]">{error}</p>
    {:else if !policy}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
    {:else if !policy.enabled}
        <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off Git sync on this server.</p>
    {:else}
        <GitLinkForm
            genericHostsEnabled={policy.generic_hosts_enabled}
            intro="Create a project from a folder in a Git repository and keep it in sync. Only document sources, bibliographies, images and fonts are copied. Everything else stays in the repository untouched, so large repositories are fine."
            submitLabel="Import project"
            showReplaceOption={false}
            ready={projectName.trim().length > 0}
            onpreview={previewGitImport}
            onsubmit={importProject}
            bind:repoUrl
        >
            <label class="flex flex-col gap-1.5 text-xs">
                <span class="font-medium text-[var(--color-ink-muted)]">Project name</span>
                <input
                    bind:value={projectName}
                    oninput={() => (nameEdited = projectName.trim().length > 0)}
                    placeholder="My paper"
                    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                />
            </label>
        </GitLinkForm>
    {/if}

    {#snippet footer()}
        <button onclick={onclose} class="rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] transition hover:bg-[var(--color-surface-sunken)]">Cancel</button>
    {/snippet}
</Modal>
