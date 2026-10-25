<script lang="ts">
    import ThemePicker from '#lib/components/ThemePicker.svelte';
    import AccessibilitySettings from '#lib/components/settings/AccessibilitySettings.svelte';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ThemeCollection from '#lib/components/themes/ThemeCollection.svelte';
    import { customThemesStore } from '#lib/ts/custom-themes';
</script>

<SettingsHeader title="Appearance" description="Choose how TypstDrive looks on this browser." icon="mdi:palette-outline" />

<div class="space-y-6">
    <SettingsCard title="Theme" description="Applies to the drive, editor and preview chrome.">
        <SettingsRow label="Color theme and mode" description="Pick a palette, then light or dark.">
            <ThemePicker class="flex-wrap sm:justify-end" />
        </SettingsRow>
    </SettingsCard>

    {#if $customThemesStore.personal_enabled}
        <SettingsCard title="Your themes" description="Make your own color themes with color pickers, or upload a YAML file. Your themes follow your account to any browser.">
            <ThemeCollection scope="personal" themes={$customThemesStore.personal} emptyText="You have no themes of your own yet." />
        </SettingsCard>
    {/if}

    <AccessibilitySettings />
</div>
