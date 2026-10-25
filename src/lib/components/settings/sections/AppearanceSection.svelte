<script lang="ts">
    import ThemePicker from '#lib/components/ThemePicker.svelte';
    import AccessibilitySettings from '#lib/components/settings/AccessibilitySettings.svelte';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
</script>

<SettingsHeader title="Appearance" description="Choose how TypstDrive looks on this browser." icon="mdi:palette-outline" />

<div class="space-y-6">
    <SettingsCard title="Theme" description="Applies to the drive, editor and preview chrome.">
        <SettingsRow label="Color theme and mode" description="Pick a palette, then light or dark.">
            <ThemePicker class="flex-wrap sm:justify-end" />
        </SettingsRow>
    </SettingsCard>

    <AccessibilitySettings />
</div>
