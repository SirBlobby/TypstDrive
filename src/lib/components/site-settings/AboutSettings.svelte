<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { brandingStore, logoUrl, PROJECT_URL } from '#lib/ts/branding';
    import { fetchServerInfo, type ServerInfo } from '#lib/ts/server-info';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';

    const links = [
        { label: 'Typst Documentation', url: 'https://typst.app/docs/', icon: 'mdi:book-open-page-variant-outline' },
        { label: 'Typst Universe', url: 'https://typst.app/universe/', icon: 'mdi:earth' },
        { label: 'MiTeX, used for LaTeX documents', url: 'https://github.com/mitex-rs/mitex', icon: 'mdi:sigma' },
        { label: 'TypstDrive Documentation', url: 'https://sirblobby.github.io/TypstDrive/', icon: 'mdi:file-document-multiple-outline' },
        { label: 'GitHub Repository', url: PROJECT_URL, icon: 'mdi:github' },
        { label: 'Report an Issue', url: `${PROJECT_URL}/issues`, icon: 'mdi:bug-outline' }
    ];

    let info = $state<ServerInfo | null>(null);
    let loadError = $state('');

    let details = $derived(
        info
            ? [
                  ['Version', info.server_version],
                  ['Typst', info.typst_version],
                  ['LaTeX renderer', info.latex_package],
                  ['Database schema', String(info.database.schema_version)],
                  ['Author', 'SirBlobby'],
                  ['License', 'Apache-2.0']
              ]
            : []
    );

    onMount(async () => {
        try {
            info = await fetchServerInfo();
        } catch (err) {
            loadError = (err as Error).message;
        }
    });
</script>

<SettingsHeader title="About" description="Version information and links." icon="mdi:information-outline" />

<div class="space-y-6">
    <SettingsCard>
        <div class="flex items-center gap-4 px-5 py-4">
            <span class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-md bg-[var(--color-accent)]">
                <img src={logoUrl($brandingStore)} alt={$brandingStore.site_name} class="h-10 w-10 object-contain" />
            </span>
            <div class="min-w-0">
                <p class="truncate text-base font-semibold text-[var(--color-ink)]">{$brandingStore.site_name}</p>
                <p class="text-sm text-[var(--color-ink-muted)]">Powered by TypstDrive, a collaborative Typst editor and document manager with basic LaTeX support.</p>
            </div>
        </div>
        {#if loadError}
            <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
        {:else if !info}
            <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
        {:else}
            {#each details as [label, value] (label)}
                <SettingsRow {label}><span class="text-sm font-medium text-[var(--color-ink)]">{value}</span></SettingsRow>
            {/each}
        {/if}
    </SettingsCard>

    <SettingsCard title="Links">
        {#each links as link (link.url)}
            <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                class="flex items-center gap-3 px-5 py-3 text-sm text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-muted)]"
            >
                <Icon icon={link.icon} class="text-lg text-[var(--color-accent)]" />
                <span class="flex-1">{link.label}</span>
                <Icon icon="mdi:open-in-new" class="text-base text-[var(--color-ink-muted)]" />
            </a>
        {/each}
    </SettingsCard>

    <p class="text-xs leading-relaxed text-[var(--color-ink-muted)]">
        TypstDrive is a community application and is not affiliated with, endorsed by, or supported by the official Typst project. The Typst links above open the official Typst website in a new tab.
        LaTeX documents are converted to Typst with the MiTeX package, which the server downloads from Typst Universe the first time a LaTeX document is rendered, so it needs internet access.
    </p>
</div>
