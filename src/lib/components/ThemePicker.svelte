<script lang="ts">
    import { themeStore, darkModeStore } from '../ts/store';
    import { builtinThemeNames } from '../ts/builtin-palettes';
    import { customThemesStore, themeKey } from '../ts/custom-themes';
    import Icon from '@iconify/svelte';

    let { class: className = '' } = $props();
</script>

<div class="flex items-center gap-2 {className}">
    <select
        bind:value={$themeStore}
        aria-label="Color theme"
        class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs font-medium text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
    >
        <optgroup label="Built-in">
            {#each builtinThemeNames as name (name)}
                <option value={name}>{name}</option>
            {/each}
        </optgroup>
        {#if $customThemesStore.site.length > 0}
            <optgroup label="Site themes">
                {#each $customThemesStore.site as theme (theme.id)}
                    <option value={themeKey('site', theme.id)}>{theme.name}</option>
                {/each}
            </optgroup>
        {/if}
        {#if $customThemesStore.personal.length > 0}
            <optgroup label="Your themes">
                {#each $customThemesStore.personal as theme (theme.id)}
                    <option value={themeKey('personal', theme.id)}>{theme.name}</option>
                {/each}
            </optgroup>
        {/if}
    </select>

    <div class="flex rounded-lg bg-[var(--color-surface-sunken)] p-0.5 text-xs font-medium">
        <button
            class="flex items-center rounded-md px-2.5 py-1.5 transition
                {!$darkModeStore
                ? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
                : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
            onclick={() => ($darkModeStore = false)}
            aria-label="Light mode"
            title="Light mode"
        >
            <Icon icon="ph:sun" class="text-sm" />
        </button>
        <button
            class="flex items-center rounded-md px-2.5 py-1.5 transition
                {$darkModeStore
                ? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
                : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
            onclick={() => ($darkModeStore = true)}
            aria-label="Dark mode"
            title="Dark mode"
        >
            <Icon icon="ph:moon" class="text-sm" />
        </button>
    </div>
</div>
