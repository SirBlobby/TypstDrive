<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import ThemeCollection from '#lib/components/themes/ThemeCollection.svelte';
    import { customThemesStore, fetchThemePolicy, saveThemePolicy, type ThemePolicy } from '#lib/ts/custom-themes';

    let policy = $state<ThemePolicy | null>(null);
    let saved = $state<ThemePolicy | null>(null);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(!!policy && !!saved && policy.personal_enabled !== saved.personal_enabled);

    function apply(value: ThemePolicy) {
        policy = { ...value };
        saved = { ...value };
    }

    async function save() {
        if (!policy) return;
        saving = true;
        message = null;
        try {
            apply(await saveThemePolicy(policy));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            apply(await fetchThemePolicy());
        } catch (err) {
            loadError = (err as Error).message;
        }
    });
</script>

<SettingsHeader
    title="Themes"
    description="Add color themes everyone on this server can pick. Themes change the app's colors only; the site name, logo and accent color in Branding stay the same."
    icon="mdi:palette-outline"
/>

<div class="space-y-6">
    <SettingsCard title="Site themes" description="Shown under Site themes in everyone's theme picker. Create one with color pickers or upload a YAML file.">
        <ThemeCollection scope="site" themes={$customThemesStore.site} emptyText="No site themes yet. Everyone can pick from the built-in themes." />
    </SettingsCard>

    {#if loadError}
        <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if !policy}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
    {:else}
        <SettingsCard title="Personal themes">
            <SettingsRow label="Let people make their own themes" description="People can create, upload and use up to 20 themes of their own in Settings, Appearance. When off, their themes are kept but hidden.">
                <ToggleSwitch label="Personal themes" bind:checked={policy.personal_enabled} />
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
</div>
