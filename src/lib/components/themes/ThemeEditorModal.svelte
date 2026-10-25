<script lang="ts">
    import Icon from '@iconify/svelte';
    import { untrack } from 'svelte';
    import Modal from '../Modal.svelte';
    import ColorField from './ColorField.svelte';
    import ThemePreview from './ThemePreview.svelte';
    import { appColorFields, editorColorFields, cloneDraft, type PaletteKey, type ThemeDraft, type ThemeMode } from '#lib/ts/custom-themes';
    import { builtinThemeDraft, builtinThemeNames } from '#lib/ts/builtin-palettes';
    import { parseThemeYaml, downloadThemeYaml } from '#lib/ts/theme-yaml';
    import { focusOnMount } from '#lib/ts/focus-on-mount';

    let { title, theme, onsave, onclose }: {
        title: string;
        theme: ThemeDraft;
        onsave: (draft: ThemeDraft) => Promise<void>;
        onclose: () => void;
    } = $props();

    const modes: { id: ThemeMode; label: string; icon: string }[] = [
        { id: 'light', label: 'Light', icon: 'ph:sun' },
        { id: 'dark', label: 'Dark', icon: 'ph:moon' }
    ];

    let draft = $state(cloneDraft(untrack(() => theme)));
    let mode = $state<ThemeMode>('light');
    let saving = $state(false);
    let problem = $state('');
    let fileInput: HTMLInputElement;

    let canSave = $derived(draft.name.trim().length > 0 && !saving);

    function startFrom(event: Event) {
        const select = event.currentTarget as HTMLSelectElement;
        if (!select.value) return;
        const base = builtinThemeDraft(select.value);
        draft.light = base.light;
        draft.dark = base.dark;
        select.value = '';
    }

    async function upload(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;
        try {
            const uploaded = parseThemeYaml(await file.text());
            draft = { ...uploaded, name: uploaded.name || draft.name };
            problem = '';
        } catch (err) {
            problem = (err as Error).message;
        }
    }

    async function save(event: SubmitEvent) {
        event.preventDefault();
        if (!canSave) return;
        saving = true;
        problem = '';
        try {
            await onsave({ ...cloneDraft(draft), name: draft.name.trim() });
        } catch (err) {
            problem = (err as Error).message;
        }
        saving = false;
    }
</script>

{#snippet colorGroup(heading: string, fields: readonly { key: PaletteKey; label: string }[])}
    <fieldset class="space-y-2">
        <legend class="mb-2 text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">{heading}</legend>
        <div class="grid gap-2 sm:grid-cols-2">
            {#each fields as field (field.key)}
                <ColorField label={field.label} bind:value={draft[mode][field.key]} />
            {/each}
        </div>
    </fieldset>
{/snippet}

<Modal {title} icon="mdi:palette-outline" width="max-w-4xl" {onclose}>
    <form id="theme-editor-form" onsubmit={save} class="flex flex-col gap-5">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-end">
            <label class="flex flex-1 flex-col gap-1.5 text-xs">
                <span class="font-medium text-[var(--color-ink-muted)]">Name</span>
                <input
                    use:focusOnMount
                    bind:value={draft.name}
                    maxlength="40"
                    placeholder="Nord"
                    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                />
            </label>
            <div class="flex flex-wrap items-center gap-2">
                <select
                    onchange={startFrom}
                    aria-label="Start from a built-in theme"
                    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-2 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                >
                    <option value="">Start from...</option>
                    {#each builtinThemeNames as name (name)}
                        <option value={name}>{name}</option>
                    {/each}
                </select>
                <button type="button" onclick={() => fileInput.click()} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-2.5 py-2 text-xs text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                    <Icon icon="mdi:upload" class="text-sm" /> Upload YAML
                </button>
                <button type="button" onclick={() => downloadThemeYaml(draft)} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-2.5 py-2 text-xs text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                    <Icon icon="mdi:download" class="text-sm" /> Download YAML
                </button>
                <input bind:this={fileInput} type="file" accept=".yml,.yaml,application/yaml,text/yaml" class="hidden" onchange={upload} />
            </div>
        </div>

        <div class="flex rounded-lg bg-[var(--color-surface-sunken)] p-0.5 text-xs font-medium self-start" role="tablist" aria-label="Mode">
            {#each modes as option (option.id)}
                <button
                    type="button"
                    role="tab"
                    aria-selected={mode === option.id}
                    onclick={() => (mode = option.id)}
                    class="flex items-center gap-1.5 rounded-md px-3 py-1.5 transition {mode === option.id
                        ? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
                        : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    <Icon icon={option.icon} class="text-sm" />
                    {option.label}
                </button>
            {/each}
        </div>

        <div class="grid gap-6 md:grid-cols-[1fr_17rem]">
            <div class="space-y-5">
                {@render colorGroup('App', appColorFields)}
                {@render colorGroup('Editor', editorColorFields)}
            </div>
            <div class="md:sticky md:top-0 md:self-start">
                <p class="mb-2 text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">Preview</p>
                <ThemePreview palette={draft[mode]} />
            </div>
        </div>
    </form>

    {#snippet footer()}
        {#if problem}
            <p class="mr-auto text-xs text-[var(--color-danger)]">{problem}</p>
        {/if}
        <button class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]" onclick={onclose}>
            Cancel
        </button>
        <button
            type="submit"
            form="theme-editor-form"
            disabled={!canSave}
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
        >
            {saving ? 'Saving...' : 'Save theme'}
        </button>
    {/snippet}
</Modal>
