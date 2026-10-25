<script lang="ts">
    import Icon from '@iconify/svelte';
    import { defaultFolderColor, folderColorOptions } from '#lib/ts/folder-colors';

    let { color, onchange }: {
        color: string | null | undefined;
        onchange: (color: string | null) => void;
    } = $props();

    function choose(event: MouseEvent, value: string | null) {
        event.stopPropagation();
        if (value !== (color ?? null)) onchange(value);
    }
</script>

<div class="grid grid-cols-5 gap-1.5" role="radiogroup" aria-label="Folder color">
    {#each folderColorOptions as option (option.label)}
        {@const selected = (color ?? null) === option.value}
        <button
            role="radio"
            aria-checked={selected}
            aria-label={option.label}
            title={option.label}
            onclick={(event) => choose(event, option.value)}
            class="flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110 {selected ? 'ring-2 ring-[var(--color-ink)] ring-offset-2 ring-offset-[var(--color-surface)]' : ''}"
            style="background-color: {option.value ?? defaultFolderColor};"
        >
            {#if selected}
                <Icon icon="mdi:check" class="text-sm text-white" />
            {/if}
        </button>
    {/each}
</div>
