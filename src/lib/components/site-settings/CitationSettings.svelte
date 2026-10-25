<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import { fetchCitationAdminPolicy, saveCitationAdminPolicy, type CitationPolicy } from '#lib/ts/citations';

    let policy = $state<CitationPolicy | null>(null);
    let saved = $state<CitationPolicy | null>(null);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(!!policy && JSON.stringify(policy) !== JSON.stringify(saved));

    function apply(value: CitationPolicy) {
        policy = { ...value };
        saved = { ...value };
    }

    async function save() {
        if (!policy) return;
        saving = true;
        message = null;
        try {
            apply(await saveCitationAdminPolicy(policy));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            apply(await fetchCitationAdminPolicy());
        } catch (err) {
            loadError = (err as Error).message;
        }
    });
</script>

<SettingsHeader
    title="Citations"
    description="Citation tools contact outside services: doi.org, api.crossref.org, api.ncbi.nlm.nih.gov and api.semanticscholar.org for reference lookups, pub.orcid.org for ORCID, and api.zotero.org for Zotero sync. Turn any of them off if your server should not reach them."
    icon="ph:quotes"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <SettingsCard title="Services" description="Bibliography files that were already added or synced keep working when these are off.">
        <SettingsRow label="Reference lookups" description="Let people turn a DOI, arXiv, PubMed, PMC or Semantic Scholar ID into a BibTeX or Hayagriva entry.">
            <ToggleSwitch label="Reference lookups" bind:checked={policy.lookup_enabled} />
        </SettingsRow>
        <SettingsRow label="ORCID lookups" description="Let people fill in author names and affiliations from an ORCID iD when creating from a template.">
            <ToggleSwitch label="ORCID lookups" bind:checked={policy.orcid_enabled} />
        </SettingsRow>
        <SettingsRow label="Zotero sync" description="Let people connect Zotero and sync libraries and collections into bibliography files.">
            <ToggleSwitch label="Zotero sync" bind:checked={policy.zotero_enabled} />
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
