<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import { fetchGitAdminPolicy, saveGitAdminPolicy, type GitPolicy } from '#lib/ts/git';

    let policy = $state<GitPolicy | null>(null);
    let saved = $state<GitPolicy | null>(null);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(!!policy && JSON.stringify(policy) !== JSON.stringify(saved));

    function apply(value: GitPolicy) {
        policy = { ...value };
        saved = { ...value };
    }

    async function save() {
        if (!policy) return;
        saving = true;
        message = null;
        try {
            apply(await saveGitAdminPolicy(policy));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            apply(await fetchGitAdminPolicy());
        } catch (err) {
            loadError = (err as Error).message;
        }
    });
</script>

<SettingsHeader
    title="Git"
    description="Projects can be linked to a folder in a Git repository. TypstDrive only copies the document files it needs and never clones whole repositories. GitHub and GitLab are reached through their web APIs; other servers use the git program with a small, disposable cache."
    icon="mdi:git"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <SettingsCard title="Git sync" description="Project files that were already imported stay in place when these are off.">
        <SettingsRow label="Git sync" description="Let project owners link a repository and let editors pull and push changes.">
            <ToggleSwitch label="Git sync" bind:checked={policy.enabled} />
        </SettingsRow>
        <SettingsRow label="Other Git servers" description="Allow repositories outside GitHub and GitLab, such as Gitea, Forgejo or Codeberg. Needs git installed on the server.">
            <ToggleSwitch label="Other Git servers" bind:checked={policy.generic_hosts_enabled} />
        </SettingsRow>
        {#snippet footer()}
            {#if message}
                <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
            {/if}
            <button onclick={() => saved && apply(saved)} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
            <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                {saving ? 'Saving...' : 'Save settings'}
            </button>
        {/snippet}
    </SettingsCard>
{/if}
