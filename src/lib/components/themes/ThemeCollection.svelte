<script lang="ts">
    import Icon from '@iconify/svelte';
    import ThemeEditorModal from './ThemeEditorModal.svelte';
    import ConfirmModal from '../ConfirmModal.svelte';
    import { themeStore } from '#lib/ts/store';
    import { builtinThemeDraft } from '#lib/ts/builtin-palettes';
    import {
        customThemesStore,
        findCustomTheme,
        cloneDraft,
        saveTheme,
        deleteTheme,
        themeKey,
        type CustomTheme,
        type ThemeDraft,
        type ThemeScope
    } from '#lib/ts/custom-themes';
    import { parseThemeYaml, downloadThemeYaml } from '#lib/ts/theme-yaml';

    let { scope, themes, emptyText }: {
        scope: ThemeScope;
        themes: CustomTheme[];
        emptyText: string;
    } = $props();

    let editing = $state<{ id?: string; draft: ThemeDraft } | null>(null);
    let removing = $state<CustomTheme | null>(null);
    let problem = $state('');
    let fileInput: HTMLInputElement;

    const iconButtonClass = 'rounded p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]';

    function currentThemeDraft(): ThemeDraft {
        const custom = findCustomTheme($customThemesStore, $themeStore);
        return { ...(custom ? cloneDraft(custom) : builtinThemeDraft($themeStore)), name: '' };
    }

    async function upload(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;
        try {
            editing = { draft: parseThemeYaml(await file.text()) };
            problem = '';
        } catch (err) {
            problem = (err as Error).message;
        }
    }

    async function save(draft: ThemeDraft) {
        const editingId = editing?.id;
        const saved = await saveTheme(scope, draft, editingId);
        editing = null;
        if (!editingId && scope === 'personal') $themeStore = themeKey(scope, saved.id);
    }

    async function confirmRemove() {
        const theme = removing;
        removing = null;
        if (!theme) return;
        try {
            await deleteTheme(scope, theme.id);
            problem = '';
        } catch (err) {
            problem = (err as Error).message;
        }
    }
</script>

<div class="flex flex-wrap items-center gap-2 px-5 py-3">
    <button onclick={() => (editing = { draft: currentThemeDraft() })} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">
        <Icon icon="mdi:plus" class="text-sm" /> New theme
    </button>
    <button onclick={() => fileInput.click()} class="flex items-center gap-1.5 rounded-md border border-[var(--color-line)] px-3 py-1.5 text-xs text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
        <Icon icon="mdi:upload" class="text-sm" /> Upload YAML
    </button>
    <input bind:this={fileInput} type="file" accept=".yml,.yaml,application/yaml,text/yaml" class="hidden" onchange={upload} />
    {#if problem}
        <p class="text-xs text-[var(--color-danger)]">{problem}</p>
    {/if}
</div>

{#if themes.length === 0}
    <p class="px-5 py-4 text-sm text-[var(--color-ink-muted)]">{emptyText}</p>
{:else}
    <ul class="divide-y divide-[var(--color-line)]">
        {#each themes as theme (theme.id)}
            {@const key = themeKey(scope, theme.id)}
            <li class="flex items-center gap-3 px-5 py-3">
                <div class="flex flex-shrink-0 overflow-hidden rounded border border-[var(--color-line)]">
                    {#each [theme.light, theme.dark] as palette, index (index)}
                        <span class="flex h-8 w-8 items-end justify-end p-1" style="background: {palette.surface};">
                            <span class="h-2.5 w-2.5 rounded-full" style="background: {palette.accent};"></span>
                        </span>
                    {/each}
                </div>
                <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium text-[var(--color-ink)]">{theme.name}</p>
                    {#if $themeStore === key}
                        <p class="text-xs text-[var(--color-accent)]">In use on this browser</p>
                    {/if}
                </div>
                <div class="flex items-center gap-0.5">
                    {#if $themeStore !== key}
                        <button onclick={() => ($themeStore = key)} class="mr-1 rounded-md border border-[var(--color-line)] px-2.5 py-1 text-xs text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">Use</button>
                    {/if}
                    <button onclick={() => (editing = { id: theme.id, draft: cloneDraft(theme) })} class={iconButtonClass} title="Edit" aria-label="Edit {theme.name}">
                        <Icon icon="mdi:pencil-outline" class="text-base" />
                    </button>
                    <button onclick={() => downloadThemeYaml(theme)} class={iconButtonClass} title="Download YAML" aria-label="Download {theme.name}">
                        <Icon icon="mdi:download" class="text-base" />
                    </button>
                    <button onclick={() => (removing = theme)} class="{iconButtonClass} hover:text-[var(--color-danger)]" title="Delete" aria-label="Delete {theme.name}">
                        <Icon icon="mdi:trash-can-outline" class="text-base" />
                    </button>
                </div>
            </li>
        {/each}
    </ul>
{/if}

{#if editing}
    <ThemeEditorModal
        title={editing.id ? 'Edit theme' : 'New theme'}
        theme={editing.draft}
        onsave={save}
        onclose={() => (editing = null)}
    />
{/if}

{#if removing}
    <ConfirmModal
        title="Delete theme"
        message={scope === 'site'
            ? `Delete ${removing.name}? People using it switch back to the default theme.`
            : `Delete ${removing.name}? This cannot be undone.`}
        onconfirm={confirmRemove}
        onclose={() => (removing = null)}
    />
{/if}
