<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import { draftToQuota, quotaToDraft, type QuotaDraft } from '$lib/ts/storage-quota';
    import type { PackagePolicy } from '$lib/ts/packages';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '$lib/components/settings/ToggleSwitch.svelte';
    import QuotaInput from './QuotaInput.svelte';

    let policy = $state<PackagePolicy | null>(null);
    let saved = $state<PackagePolicy | null>(null);
    let sizeDraft = $state<QuotaDraft>({ amount: '0', unit: 'MB' });
    let versionsText = $state('0');
    let loading = $state(true);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let maxVersions = $derived(Number(String(versionsText ?? '').trim()));
    let maxSize = $derived(draftToQuota(sizeDraft));
    let pendingPolicy = $derived<PackagePolicy | null>(
        policy && maxSize !== null && Number.isInteger(maxVersions) && maxVersions >= 0
            ? { ...policy, max_package_size_bytes: maxSize, max_versions_per_package: maxVersions }
            : null
    );
    let hasChanges = $derived.by(() => {
        if (!saved || !policy) return false;
        if (!pendingPolicy) return true;
        return JSON.stringify(pendingPolicy) !== JSON.stringify(saved);
    });

    function applyPolicy(value: PackagePolicy) {
        policy = { ...value };
        saved = { ...value };
        sizeDraft = value.max_package_size_bytes > 0 ? quotaToDraft(value.max_package_size_bytes) : { amount: '0', unit: 'MB' };
        versionsText = String(value.max_versions_per_package);
    }

    async function load() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/admin/package-policy');
            if (res.ok) applyPolicy(await res.json());
            else loadError = (await res.text()) || 'Failed to load settings.';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function save() {
        if (!pendingPolicy) {
            message = { kind: 'error', text: 'Enter valid limits.' };
            return;
        }
        saving = true;
        message = null;
        try {
            const res = await fetch('/api/admin/package-policy', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(pendingPolicy)
            });
            if (res.ok) {
                applyPolicy(await res.json());
                message = { kind: 'success', text: 'Settings saved.' };
            } else {
                message = { kind: 'error', text: (await res.text()) || 'Failed to save settings.' };
            }
        } catch {
            message = { kind: 'error', text: 'Network error.' };
        }
        saving = false;
    }

    function discard() {
        if (saved) applyPolicy(saved);
        message = null;
    }

    onMount(load);
</script>

<SettingsHeader
    title="Packages"
    description="Control who can publish Typst packages to this instance and how large they can be. Administrators can always publish and delete packages."
    icon="mdi:package-variant-closed"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if loading || !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Publishing" description="Who can publish new packages and versions.">
            <SettingsRow label="Allow package publishing" description="When off, members cannot publish packages or new versions. Existing packages stay available.">
                <ToggleSwitch label="Allow package publishing" bind:checked={policy.publishing_enabled} />
            </SettingsRow>
            <SettingsRow label="Only administrators can publish" description="Members can still import packages but cannot publish their own.">
                <ToggleSwitch label="Only administrators can publish" bind:checked={policy.admins_only} disabled={!policy.publishing_enabled} />
            </SettingsRow>
            <SettingsRow label="Owners can delete their packages" description="When off, only administrators can delete packages.">
                <ToggleSwitch label="Owners can delete their packages" bind:checked={policy.allow_owner_delete} />
            </SettingsRow>
        </SettingsCard>

        <SettingsCard title="Private packages" description="Private packages can only be used by their owner and the people and team spaces they are shared with.">
            <SettingsRow label="Allow private packages" description="When off, members can only publish public packages. Packages that are already private stay private.">
                <ToggleSwitch label="Allow private packages" bind:checked={policy.allow_private_packages} />
            </SettingsRow>
            <SettingsRow label="Default for new packages" description="Who can use a package when it is first published, unless the publisher picks otherwise." forId="default-package-visibility">
                <select id="default-package-visibility" bind:value={policy.default_visibility} class="{inputClass} sm:!max-w-40">
                    <option value="public">Public</option>
                    <option value="private">Private</option>
                </select>
            </SettingsRow>
        </SettingsCard>

        <SettingsCard title="Limits" description="Applies to every publish, including administrators. Use 0 for no limit.">
            <SettingsRow label="Maximum package size" description="Total size of all files in one published version." forId="max-package-size">
                <QuotaInput id="max-package-size" bind:draft={sizeDraft} placeholder="0" />
            </SettingsRow>
            <SettingsRow label="Maximum versions per package" description="Publishing stops once a package reaches this many versions." forId="max-package-versions">
                <input id="max-package-versions" type="number" min="0" step="1" bind:value={versionsText} placeholder="0" class="{inputClass} sm:!max-w-32" />
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
    </div>
{/if}
