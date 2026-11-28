<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/ui/settings';
    import { fetchLatexAdminPolicy, saveLatexAdminPolicy, type LatexPolicy } from '#lib/ts/preview/latex-policy';
    import { fetchServerInfo, type ServerInfo } from '#lib/ts/site/server-info';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';

    let policy = $state<LatexPolicy | null>(null);
    let saved = $state<LatexPolicy | null>(null);
    let engine = $state<ServerInfo['latex'] | null>(null);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(!!policy && !!saved && policy.enabled !== saved.enabled);

    function apply(value: LatexPolicy) {
        policy = { ...value };
        saved = { ...value };
    }

    async function save() {
        if (!policy) return;
        saving = true;
        message = null;
        try {
            apply(await saveLatexAdminPolicy(policy));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            apply(await fetchLatexAdminPolicy());
        } catch (err) {
            loadError = (err as Error).message;
        }
        fetchServerInfo()
            .then((info) => (engine = info.latex))
            .catch(() => {});
    });
</script>

<SettingsHeader
    title="LaTeX"
    description="LaTeX documents and projects are compiled on this server with Tectonic, which downloads LaTeX packages from the internet the first time they are used."
    icon="mdi:sigma"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="LaTeX support">
            <SettingsRow
                label="Allow LaTeX"
                description="When this is off, nobody can create LaTeX documents or projects, existing ones stay in the drive but do not compile, and the API does not render LaTeX. Typst is not affected."
            >
                <ToggleSwitch label="Allow LaTeX" bind:checked={policy.enabled} />
            </SettingsRow>
            {#if saved && !saved.enabled}
                <p class="flex items-start gap-2 px-5 py-3 text-xs text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:information-outline" class="mt-0.5 shrink-0 text-sm" />
                    LaTeX documents can still be opened, edited, shared and downloaded as .tex files. Their preview shows that LaTeX is turned off.
                </p>
            {/if}
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

        {#if engine}
            <SettingsCard title="Engine">
                <SettingsRow label="Tectonic"><span class="text-sm font-medium text-[var(--color-ink)]">{engine.engine ?? 'Not available'}</span></SettingsRow>
                <SettingsRow label="Sandbox"><span class="text-sm font-medium text-[var(--color-ink)]">{engine.sandbox ?? engine.problem ?? 'Not available'}</span></SettingsRow>
            </SettingsCard>
        {/if}
    </div>
{/if}
