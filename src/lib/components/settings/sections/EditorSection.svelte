<script lang="ts">
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '$lib/components/settings/ToggleSwitch.svelte';
    import KeybindsCard from '$lib/components/settings/KeybindsCard.svelte';
    import { inputClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import { editorFonts, findEditorFont, loadEditorFont } from '$lib/ts/editor-fonts';
    import {
        defaultEditorSettings,
        editorSettingsStore,
        MAX_FONT_SIZE,
        MIN_FONT_SIZE,
        spellCheckLanguages,
        updateEditorSettings
    } from '$lib/ts/editor-settings';
    import { minimapSizeOptions, type MinimapSize } from '$lib/ts/minimap/minimap-sizes';

    const sampleText = '= Introduction\n#let total = 42\nThe quick brown fox jumps over the lazy dog.';

    let previewFont = $derived(findEditorFont($editorSettingsStore.fontFamily));

    $effect(() => {
        loadEditorFont($editorSettingsStore.fontFamily);
    });
</script>

<SettingsHeader title="Editor" description="Customize the code editor. Saved to this browser." icon="mdi:code-braces" />

<div class="space-y-6">
    <SettingsCard title="Text">
        <SettingsRow label="Font size" description="{MIN_FONT_SIZE} to {MAX_FONT_SIZE} pixels." forId="editor-font-size">
            <input
                id="editor-font-size"
                type="range"
                min={MIN_FONT_SIZE}
                max={MAX_FONT_SIZE}
                value={$editorSettingsStore.fontSize}
                oninput={(event) => updateEditorSettings({ fontSize: Number(event.currentTarget.value) })}
                class="w-40 accent-[var(--color-accent)]"
            />
            <span class="w-12 text-right text-sm tabular-nums text-[var(--color-ink)]">{$editorSettingsStore.fontSize}px</span>
        </SettingsRow>
        <SettingsRow label="Font family" description="Web fonts download the first time you use them." forId="editor-font-family">
            <select
                id="editor-font-family"
                value={$editorSettingsStore.fontFamily}
                onchange={(event) => updateEditorSettings({ fontFamily: event.currentTarget.value })}
                class={inputClass}
            >
                {#each editorFonts as font (font.name)}
                    <option value={font.name}>{font.name}</option>
                {/each}
            </select>
        </SettingsRow>
        <div class="px-5 pb-4">
            <pre
                class="overflow-x-auto rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 text-[var(--color-ink)]"
                style="font-family: {previewFont.stack}; font-size: {$editorSettingsStore.fontSize}px;">{sampleText}</pre>
        </div>
    </SettingsCard>

    <SettingsCard title="Spell checker" description="Checks your writing and skips code, math, labels and links. Right-click a marked word for suggestions.">
        <SettingsRow label="Check spelling">
            <ToggleSwitch label="Check spelling" checked={$editorSettingsStore.spellCheck} onchange={(checked) => updateEditorSettings({ spellCheck: checked })} />
        </SettingsRow>
        <SettingsRow label="Language" description="Uses the dictionaries installed in your browser." forId="spell-check-language">
            <select
                id="spell-check-language"
                value={$editorSettingsStore.spellCheckLanguage}
                disabled={!$editorSettingsStore.spellCheck}
                onchange={(event) => updateEditorSettings({ spellCheckLanguage: event.currentTarget.value })}
                class={inputClass}
            >
                {#each spellCheckLanguages as language (language.code)}
                    <option value={language.code}>{language.label}</option>
                {/each}
            </select>
        </SettingsRow>
    </SettingsCard>

    <SettingsCard title="Layout">
        <SettingsRow label="Line numbers" description="Show line numbers next to the code.">
            <ToggleSwitch label="Line numbers" checked={$editorSettingsStore.lineNumbers} onchange={(checked) => updateEditorSettings({ lineNumbers: checked })} />
        </SettingsRow>
        <SettingsRow label="Minimap" description="Show an overview of the whole file on the right side.">
            <ToggleSwitch label="Minimap" checked={$editorSettingsStore.minimap} onchange={(checked) => updateEditorSettings({ minimap: checked })} />
        </SettingsRow>
        <SettingsRow label="Minimap size" description="Width of the minimap and how large its lines are drawn." forId="minimap-size">
            <select
                id="minimap-size"
                value={$editorSettingsStore.minimapSize}
                disabled={!$editorSettingsStore.minimap}
                onchange={(event) => updateEditorSettings({ minimapSize: event.currentTarget.value as MinimapSize })}
                class={inputClass}
            >
                {#each minimapSizeOptions as option (option.value)}
                    <option value={option.value}>{option.label}</option>
                {/each}
            </select>
        </SettingsRow>
        {#snippet footer()}
            <button type="button" onclick={() => editorSettingsStore.set(defaultEditorSettings)} class={secondaryButtonClass}>Reset to defaults</button>
        {/snippet}
    </SettingsCard>

    <KeybindsCard />
</div>
