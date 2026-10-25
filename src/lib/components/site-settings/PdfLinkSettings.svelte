<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import PdfLinkRow from '#lib/components/settings/PdfLinkRow.svelte';
    import { adminRemovePdfLink, fetchPdfLinkOverview, savePdfLinkPolicy, type PdfLink, type PdfLinkPolicy } from '#lib/ts/pdf-links';

    let policy = $state<PdfLinkPolicy | null>(null);
    let saved = $state<PdfLinkPolicy | null>(null);
    let limitText = $state('0');
    let links = $state<PdfLink[]>([]);
    let filter = $state('');
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let limit = $derived(Number(String(limitText ?? '').trim()));
    let pendingPolicy = $derived<PdfLinkPolicy | null>(
        policy && Number.isInteger(limit) && limit >= 0 ? { ...policy, max_links_per_user: limit } : null
    );
    let hasChanges = $derived(!!saved && JSON.stringify(pendingPolicy) !== JSON.stringify(saved));
    let filteredLinks = $derived.by(() => {
        const query = filter.trim().toLowerCase();
        if (!query) return links;
        return links.filter((link) =>
            [link.slug, link.item_name, link.owner_username ?? ''].some((value) => value.toLowerCase().includes(query))
        );
    });

    function applyPolicy(value: PdfLinkPolicy) {
        policy = { ...value };
        saved = { ...value };
        limitText = String(value.max_links_per_user);
    }

    async function load() {
        loadError = '';
        try {
            const overview = await fetchPdfLinkOverview();
            applyPolicy(overview.policy);
            links = overview.links;
        } catch (err) {
            loadError = (err as Error).message;
        }
    }

    async function save() {
        if (!pendingPolicy) {
            message = { kind: 'error', text: 'Enter a valid limit.' };
            return;
        }
        saving = true;
        message = null;
        try {
            applyPolicy(await savePdfLinkPolicy(pendingPolicy));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    function discard() {
        if (saved) applyPolicy(saved);
        message = null;
    }

    async function remove(link: PdfLink) {
        try {
            await adminRemovePdfLink(link.id);
            links = links.filter((existing) => existing.id !== link.id);
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
    }

    onMount(load);
</script>

<SettingsHeader
    title="PDF Links"
    description="Public URLs at /pdf/name that always serve the latest compiled PDF of a document or project. Only owners can create them, and anyone with the URL can download the PDF."
    icon="mdi:file-pdf-box"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Policy" description="Turning links off stops every link from serving PDFs. Links are kept and work again when turned back on.">
            <SettingsRow label="Allow public PDF links" description="When off, links return not found and members cannot create or rename links.">
                <ToggleSwitch label="Allow public PDF links" bind:checked={policy.enabled} />
            </SettingsRow>
            <SettingsRow label="Maximum links per user" description="Use 0 for no limit. Existing links over the limit are kept." forId="max-pdf-links">
                <input id="max-pdf-links" type="number" min="0" step="1" bind:value={limitText} placeholder="0" class="{inputClass} sm:!max-w-32" />
            </SettingsRow>
            {#snippet footer()}
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={discard} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
                <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                    <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                    {saving ? 'Saving...' : 'Save settings'}
                </button>
            {/snippet}
        </SettingsCard>

        <SettingsCard title="All links" description={`${links.length} ${links.length === 1 ? 'link' : 'links'} on this instance.`}>
            {#if links.length > 0}
                <div class="px-5 py-3">
                    <input bind:value={filter} placeholder="Filter by name, document or owner" class="{inputClass} sm:!max-w-sm" />
                </div>
            {/if}
            {#if filteredLinks.length === 0}
                <p class="px-5 py-8 text-center text-sm text-[var(--color-ink-muted)]">{links.length === 0 ? 'No public PDF links yet.' : 'No links match.'}</p>
            {:else}
                {#each filteredLinks as link (link.id)}
                    <PdfLinkRow {link} showOwner ondelete={remove} />
                {/each}
            {/if}
        </SettingsCard>
    </div>
{/if}
