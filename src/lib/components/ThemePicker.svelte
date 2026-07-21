<script lang="ts">
    import { themeStore, darkModeStore } from '../ts/store';
    import { themes } from '../ts/themes';
    import Icon from '@iconify/svelte';

    let { class: className = '' } = $props();

    const themeNames = Object.keys(themes);
</script>

<div class="flex items-center gap-2 {className}">
    <div class="flex rounded-lg bg-[var(--color-surface-sunken)] p-0.5 text-xs font-medium">
        {#each themeNames as name}
            <button
                class="flex items-center gap-1.5 rounded-md px-2.5 py-1.5 transition
                    {$themeStore === name
                    ? 'bg-[var(--color-surface)] text-[var(--color-accent)] shadow-sm'
                    : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                onclick={() => ($themeStore = name)}
            >
                <Icon icon={themes[name].icon} class="text-sm" />
                {name}
            </button>
        {/each}
    </div>

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
