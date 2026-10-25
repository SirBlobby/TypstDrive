<script lang="ts">
    import Icon from '@iconify/svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import KeybindRecorder from '#lib/components/settings/KeybindRecorder.svelte';
    import { secondaryButtonClass } from '#lib/ts/settings-ui';
    import { keybindActions } from '#lib/ts/keybinds/keybind-actions';
    import { conflictingActions, defaultKeybinds, keybindsStore, resetKeybinds, setKeybind } from '#lib/ts/keybinds/keybinds-store';

    const conflicts = $derived(conflictingActions($keybindsStore));
</script>

<SettingsCard
    title="Keybinds"
    description="Click a shortcut, then press the new key combination. Escape cancels and Backspace removes the shortcut."
>
    {#each keybindActions as action (action.id)}
        {@const isDefault = $keybindsStore[action.id] === defaultKeybinds[action.id]}
        <SettingsRow label={action.label} description={conflicts.has(action.id) ? 'Shared with another action' : undefined}>
            <KeybindRecorder
                label={action.label}
                value={$keybindsStore[action.id]}
                conflict={conflicts.has(action.id)}
                onchange={(key) => setKeybind(action.id, key)}
            />
            <button
                type="button"
                class="rounded-md p-1.5 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] disabled:invisible"
                disabled={isDefault}
                title="Restore default"
                aria-label="Restore default shortcut for {action.label}"
                onclick={() => setKeybind(action.id, defaultKeybinds[action.id])}
            >
                <Icon icon="mdi:restore" class="text-lg" />
            </button>
        </SettingsRow>
    {/each}
    {#snippet footer()}
        <button type="button" onclick={resetKeybinds} class={secondaryButtonClass}>Reset keybinds</button>
    {/snippet}
</SettingsCard>
