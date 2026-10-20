<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';

    let savedLimit = $state<number | null>(null);
    let limitDraft = $state('0');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let parsedLimit = $derived(/^\d+$/.test(limitDraft.trim()) ? Number(limitDraft.trim()) : null);
    let changed = $derived(savedLimit !== null && parsedLimit !== savedLimit);

    function removalSummary(removed: number) {
        if (removed === 0) return 'Version limit saved.';
        return `Version limit saved. Removed ${removed} old ${removed === 1 ? 'version' : 'versions'}.`;
    }

    async function load() {
        const res = await fetch('/api/admin/storage/version-history');
        if (!res.ok) return;
        const data = await res.json();
        savedLimit = data.max_versions;
        limitDraft = String(data.max_versions);
    }

    async function save() {
        if (parsedLimit === null) {
            message = { kind: 'error', text: 'Enter a whole number.' };
            return;
        }
        if (parsedLimit > 0 && savedLimit !== null && (savedLimit === 0 || parsedLimit < savedLimit)) {
            const confirmed = confirm(`Documents with more than ${parsedLimit} versions will lose their oldest versions. Continue?`);
            if (!confirmed) return;
        }
        saving = true;
        message = null;
        try {
            const res = await fetch('/api/admin/storage/version-history', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ max_versions: parsedLimit })
            });
            if (res.ok) {
                const data = await res.json();
                savedLimit = data.max_versions;
                limitDraft = String(data.max_versions);
                message = { kind: 'success', text: removalSummary(data.removed_versions) };
            } else {
                message = { kind: 'error', text: (await res.text()) || 'Failed to save the version limit.' };
            }
        } catch {
            message = { kind: 'error', text: 'Network error.' };
        }
        saving = false;
    }

    onMount(load);
</script>

<SettingsCard title="Version history" description="Auto saved versions made within 15 minutes of each other are merged into one. Use 0 to keep every version.">
    <SettingsRow label="Maximum versions per document" description="When a document goes over the limit, its oldest versions are deleted." forId="max-versions">
        <input id="max-versions" type="text" inputmode="numeric" bind:value={limitDraft} placeholder="0" class="{inputClass} !w-32 text-right tabular-nums" />
    </SettingsRow>
    {#snippet footer()}
        {#if message}
            <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
        {/if}
        <button onclick={() => savedLimit !== null && (limitDraft = String(savedLimit))} disabled={!changed || saving} class={secondaryButtonClass}>Discard</button>
        <button onclick={save} disabled={!changed || saving} class={primaryButtonClass}>
            <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
            {saving ? 'Saving...' : 'Save limit'}
        </button>
    {/snippet}
</SettingsCard>
