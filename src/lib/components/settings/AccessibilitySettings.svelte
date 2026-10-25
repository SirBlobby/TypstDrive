<script lang="ts">
    import { darkModeStore, highContrastStore, textSizeStore, type TextSize } from '#lib/ts/store';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';

    const textSizes: { value: TextSize; label: string; title: string }[] = [
        { value: 'small', label: 'A−', title: 'Smaller text' },
        { value: 'default', label: 'A', title: 'Default text size' },
        { value: 'large', label: 'A+', title: 'Larger text' }
    ];
</script>

{#snippet toggle(label: string, checked: boolean, onToggle: () => void)}
    <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onclick={onToggle}
        class="relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors {checked ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface-sunken)]'}"
    >
        <span class="inline-block h-5 w-5 rounded-full bg-white shadow transition-transform {checked ? 'translate-x-5' : 'translate-x-0.5'}"></span>
    </button>
{/snippet}

<SettingsCard title="Accessibility" description="Saved to this browser.">
    <SettingsRow label="Text size" description="Scale text across the drive, editor and settings.">
        <div class="flex rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-0.5 text-sm font-semibold" role="radiogroup" aria-label="Text size">
            {#each textSizes as size}
                <button
                    type="button"
                    role="radio"
                    aria-checked={$textSizeStore === size.value}
                    title={size.title}
                    onclick={() => ($textSizeStore = size.value)}
                    class="min-w-10 rounded-md px-3 py-1.5 transition {$textSizeStore === size.value
                        ? 'bg-[var(--color-surface)] text-[var(--color-accent)] shadow-sm'
                        : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    {size.label}
                </button>
            {/each}
        </div>
    </SettingsRow>
    <SettingsRow label="Dark mode" description="Use a dark color scheme.">
        {@render toggle('Dark mode', $darkModeStore, () => ($darkModeStore = !$darkModeStore))}
    </SettingsRow>
    <SettingsRow label="High contrast" description="Increase contrast of text, borders and backgrounds.">
        {@render toggle('High contrast', $highContrastStore, () => ($highContrastStore = !$highContrastStore))}
    </SettingsRow>
</SettingsCard>
