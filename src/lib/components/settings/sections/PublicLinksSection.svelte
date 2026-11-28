<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import PublicLinkRow from '#lib/components/settings/PublicLinkRow.svelte';
    import PromptModal from '#lib/components/PromptModal.svelte';
    import {
        anyFormatAllowed,
        fetchOwnPublicLinks,
        normalizeSlug,
        removePublicLink,
        savePublicLink,
        settingsOf,
        type OwnPublicLinks,
        type PublicLink
    } from '#lib/ts/sharing/public-links';

    let data = $state<OwnPublicLinks | null>(null);
    let loadError = $state('');
    let actionError = $state('');
    let linkToRename = $state<PublicLink | null>(null);

    let linksAllowed = $derived(!!data && anyFormatAllowed(data.policy));

    async function load() {
        loadError = '';
        try {
            data = await fetchOwnPublicLinks();
        } catch (err) {
            loadError = (err as Error).message;
        }
    }

    async function remove(link: PublicLink) {
        actionError = '';
        try {
            await removePublicLink(link.kind, link.item_id);
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
            const updated = await savePublicLink(link.kind, link.item_id, { ...settingsOf(link), slug: normalizeSlug(slug) });
            data = { ...data, links: data.links.map((existing) => (existing.id === updated.id ? updated : existing)) };
        } catch (err) {
            actionError = (err as Error).message;
        }
    }

    onMount(load);
</script>

<SettingsHeader
    title="Public Links"
    description="Stable public URLs that always show the latest compiled version of a document or project, as a PDF, as slides or as a password-protected speaker view. Create them from the Share dialog."
    icon="mdi:earth"
/>

{#if data && !linksAllowed}
    <p class="mb-4 flex items-center gap-2 rounded-md border border-amber-500/40 bg-amber-500/10 px-4 py-3 text-sm text-[var(--color-ink)]">
        <Icon icon="mdi:alert-outline" class="text-lg text-amber-500" />
        An administrator has turned off public links. Your links do not work until they are turned back on.
    </p>
{/if}

{#if actionError}
    <p class="mb-4 text-sm text-[var(--color-danger)]">{actionError}</p>
{/if}

<SettingsCard
    title="Your links"
    description={data && data.policy.max_links_per_user > 0 ? `${data.links.length} of ${data.policy.max_links_per_user} links used.` : undefined}
>
    {#if loadError}
        <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if !data}
        <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading links...</p>
    {:else if data.links.length === 0}
        <div class="flex flex-col items-center px-5 py-12 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:link-variant-off" class="mb-2 text-4xl opacity-50" />
            <p class="text-sm">You have no public links.</p>
        </div>
    {:else}
        {#each data.links as link (link.id)}
            <PublicLinkRow {link} onrename={linksAllowed ? (target) => (linkToRename = target) : undefined} ondelete={remove} />
        {/each}
    {/if}
</SettingsCard>

{#if linkToRename}
    <PromptModal
        title="Rename public links"
        label="Link name"
        icon="mdi:earth"
        value={linkToRename.slug}
        confirmLabel="Rename"
        onsubmit={rename}
        onclose={() => (linkToRename = null)}
    />
{/if}
