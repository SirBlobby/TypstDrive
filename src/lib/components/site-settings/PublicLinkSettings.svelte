<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/ui/settings';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import PublicLinkRow from '#lib/components/settings/PublicLinkRow.svelte';
    import {
        adminRemovePublicLink,
        fetchPublicLinkOverview,
        savePublicLinkPolicy,
        type PublicLink,
        type PublicLinkPolicy
    } from '#lib/ts/sharing/public-links';

    let policy = $state<PublicLinkPolicy | null>(null);
    let saved = $state<PublicLinkPolicy | null>(null);
    let limitText = $state('0');
    let links = $state<PublicLink[]>([]);
    let filter = $state('');
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let limit = $derived(Number(String(limitText ?? '').trim()));
    let pendingPolicy = $derived<PublicLinkPolicy | null>(
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

    function applyPolicy(value: PublicLinkPolicy) {
        policy = { ...value };
        saved = { ...value };
        limitText = String(value.max_links_per_user);
    }

    async function load() {
        loadError = '';
        try {
            const overview = await fetchPublicLinkOverview();
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
            applyPolicy(await savePublicLinkPolicy(pendingPolicy));
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

    async function remove(link: PublicLink) {
        try {
            await adminRemovePublicLink(link.id);
            links = links.filter((existing) => existing.id !== link.id);
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
    }

    onMount(load);
</script>

<SettingsHeader
    title="Public Links"
    description="Public URLs that always show the latest compiled version of a document or project: /pdf/name for the PDF, /slides/name to present the slides, and /slides/name/speaker for a password-protected speaker view with notes. Only owners can create them."
    icon="mdi:earth"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Policy" description="Turning a format off stops every link from serving it. Links keep their settings and work again when it is turned back on.">
            <SettingsRow label="Allow public PDFs" description="Anyone with the link can download the latest PDF.">
                <ToggleSwitch label="Allow public PDFs" bind:checked={policy.pdf_enabled} />
            </SettingsRow>
            <SettingsRow label="Allow public slides" description="Anyone with the link can present the slides. Speaker notes are never included.">
                <ToggleSwitch label="Allow public slides" bind:checked={policy.slides_enabled} />
            </SettingsRow>
            <SettingsRow label="Allow public speaker views" description="Slides with speaker notes, behind a password the owner sets.">
                <ToggleSwitch label="Allow public speaker views" bind:checked={policy.speaker_enabled} />
            </SettingsRow>
            <SettingsRow label="Maximum links per user" description="Each document or project counts once, whatever formats it has. Use 0 for no limit. Existing links over the limit are kept." forId="max-public-links">
                <input id="max-public-links" type="number" min="0" step="1" bind:value={limitText} placeholder="0" class="{inputClass} sm:!max-w-32" />
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
                <p class="px-5 py-8 text-center text-sm text-[var(--color-ink-muted)]">{links.length === 0 ? 'No public links yet.' : 'No links match.'}</p>
            {:else}
                {#each filteredLinks as link (link.id)}
                    <PublicLinkRow {link} showOwner ondelete={remove} />
                {/each}
            {/if}
        </SettingsCard>
    </div>
{/if}
