<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { Package, PackageVersion } from '$lib/ts/packages';

    let { pkg, canDelete, onDelete }: {
        pkg: Package;
        canDelete: boolean;
        onDelete: (pkg: Package) => void;
    } = $props();

    let showVersions = $state(false);
    let versions = $state<PackageVersion[] | null>(null);
    let versionsLoading = $state(false);
    let copiedVersion = $state('');

    function importSnippet(version: string) {
        return `#import "@typstdrive/${pkg.name}:${version}": *`;
    }

    function formatDate(date: string) {
        return new Date(date.endsWith('Z') ? date : date + 'Z').toLocaleDateString(undefined, {
            month: 'short',
            day: 'numeric',
            year: 'numeric'
        });
    }

    async function copySnippet(version: string) {
        await navigator.clipboard.writeText(importSnippet(version));
        copiedVersion = version;
        setTimeout(() => (copiedVersion = ''), 2000);
    }

    async function toggleVersions() {
        showVersions = !showVersions;
        if (!showVersions || versions !== null) return;
        versionsLoading = true;
        try {
            const res = await fetch(`/api/packages/${pkg.name}`);
            versions = res.ok ? await res.json() : [];
        } catch {
            versions = [];
        }
        versionsLoading = false;
    }

    let latestVersion = $derived(pkg.latest_version ?? '0.1.0');
</script>

<article class="bg-[var(--color-surface)] rounded-xl border border-[var(--color-line)] shadow-sm flex flex-col hover:border-[var(--color-accent)] transition-colors">
    <div class="p-5 flex-1">
        <div class="flex items-start gap-3">
            <div class="flex items-center justify-center w-11 h-11 rounded-lg bg-purple-50 dark:bg-purple-500/10 flex-shrink-0">
                <Icon icon="mdi:package-variant-closed" class="text-2xl text-purple-500" />
            </div>
            <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2 flex-wrap">
                    <h3 class="font-semibold text-[var(--color-ink)] truncate" title={pkg.name}>{pkg.name}</h3>
                    {#if pkg.latest_version}
                        <span class="text-xs font-mono bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 px-1.5 py-0.5 rounded">v{pkg.latest_version}</span>
                    {/if}
                </div>
                <p class="text-xs text-[var(--color-ink-muted)] mt-0.5">
                    by {pkg.owner_name ?? 'unknown'} · published {formatDate(pkg.created_at)}
                </p>
            </div>
            {#if canDelete}
                <button onclick={() => onDelete(pkg)} title="Delete package" aria-label="Delete package" class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors">
                    <Icon icon="mdi:trash-can-outline" class="text-lg" />
                </button>
            {/if}
        </div>

        <p class="text-sm text-[var(--color-ink-muted)] mt-3 line-clamp-2 min-h-10">
            {pkg.description || 'No description provided.'}
        </p>

        <div class="mt-4 flex items-center gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] pl-3 pr-1 py-1">
            <code class="flex-1 min-w-0 truncate text-xs font-mono text-[var(--color-ink)]">{importSnippet(latestVersion)}</code>
            <button onclick={() => copySnippet(latestVersion)} title="Copy import" aria-label="Copy import" class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors flex-shrink-0">
                <Icon icon={copiedVersion === latestVersion ? 'mdi:check' : 'mdi:content-copy'} class="text-base {copiedVersion === latestVersion ? 'text-[var(--color-success)]' : ''}" />
            </button>
        </div>
    </div>

    <div class="border-t border-[var(--color-line)]">
        <button onclick={toggleVersions} class="w-full px-5 py-2.5 flex items-center justify-between text-xs font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors">
            <span class="flex items-center gap-1.5">
                <Icon icon="mdi:history" class="text-base" />
                Versions
            </span>
            <Icon icon={showVersions ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="text-base" />
        </button>

        {#if showVersions}
            <div class="px-5 pb-4 space-y-1">
                {#if versionsLoading}
                    <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-1.5">
                        <Icon icon="mdi:loading" class="animate-spin" /> Loading versions...
                    </p>
                {:else if versions && versions.length > 0}
                    {#each versions as version (version.id)}
                        <div class="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 hover:bg-[var(--color-surface-muted)]">
                            <span class="text-xs font-mono text-[var(--color-ink)]">v{version.version}</span>
                            <span class="text-xs text-[var(--color-ink-muted)] ml-auto">{formatDate(version.created_at)}</span>
                            <button onclick={() => copySnippet(version.version)} title="Copy import for this version" aria-label="Copy import for this version" class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors">
                                <Icon icon={copiedVersion === version.version ? 'mdi:check' : 'mdi:content-copy'} class="text-sm {copiedVersion === version.version ? 'text-[var(--color-success)]' : ''}" />
                            </button>
                        </div>
                    {/each}
                {:else}
                    <p class="text-xs text-[var(--color-ink-muted)]">No versions found.</p>
                {/if}
            </div>
        {/if}
    </div>
</article>
