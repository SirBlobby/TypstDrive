<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import PdfLinkRow from '#lib/components/settings/PdfLinkRow.svelte';
    import PromptModal from '#lib/components/PromptModal.svelte';
    import { fetchOwnPdfLinks, normalizeSlug, removePdfLink, savePdfLink, type OwnPdfLinks, type PdfLink } from '#lib/ts/pdf-links';

    let data = $state<OwnPdfLinks | null>(null);
    let loadError = $state('');
    let actionError = $state('');
    let linkToRename = $state<PdfLink | null>(null);

    async function load() {
        loadError = '';
        try {
            data = await fetchOwnPdfLinks();
        } catch (err) {
            loadError = (err as Error).message;
        }
    }

    async function remove(link: PdfLink) {
        actionError = '';
        try {
            await removePdfLink(link.kind, link.item_id);
            if (data) data = { ...data, links: data.links.filter((existing) => existing.id !== link.id) };
        } catch (err) {
            actionError = (err as Error).message;
        }
    }

    async function rename(slug: string) {
        const link = linkToRename;
        linkToRename = null;
        if (!link || !data) return;
        actionError = '';
        try {
            const updated = await savePdfLink(link.kind, link.item_id, normalizeSlug(slug));
            data = { ...data, links: data.links.map((existing) => (existing.id === updated.id ? updated : existing)) };
        } catch (err) {
            actionError = (err as Error).message;
        }
    }

    onMount(load);
</script>

<SettingsHeader
    title="PDF Links"
    description="Stable public URLs that always serve the latest compiled PDF of a document or project. Create them from the Share dialog. Anyone with the URL can download the PDF."
    icon="mdi:file-pdf-box"
/>

{#if data && !data.enabled}
    <p class="mb-4 flex items-center gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm text-[var(--color-ink)]">
        <Icon icon="mdi:alert-outline" class="text-lg text-amber-500" />
        An administrator has turned off public PDF links. Your links do not work until they are turned back on.
    </p>
{/if}

{#if actionError}
    <p class="mb-4 text-sm text-[var(--color-danger)]">{actionError}</p>
{/if}

<SettingsCard
    title="Your links"
    description={data && data.max_links_per_user > 0 ? `${data.links.length} of ${data.max_links_per_user} links used.` : undefined}
>
    {#if loadError}
        <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if !data}
        <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading links...</p>
    {:else if data.links.length === 0}
        <div class="flex flex-col items-center px-5 py-12 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:link-variant-off" class="mb-2 text-4xl opacity-50" />
            <p class="text-sm">You have no public PDF links.</p>
        </div>
    {:else}
        {#each data.links as link (link.id)}
            <PdfLinkRow {link} onrename={data.enabled ? (target) => (linkToRename = target) : undefined} ondelete={remove} />
        {/each}
    {/if}
</SettingsCard>

{#if linkToRename}
    <PromptModal
        title="Rename PDF link"
        label="Short link name"
        icon="mdi:file-pdf-box"
        value={linkToRename.slug}
        confirmLabel="Rename"
        onsubmit={rename}
        onclose={() => (linkToRename = null)}
    />
{/if}
