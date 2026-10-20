<script lang="ts">
    import Icon from '@iconify/svelte';
    import { defaultFolderColor, folderColorOptions } from '$lib/ts/folder-colors';

    let { color, onchange, buttonClass }: {
        color: string | null | undefined;
        onchange: (color: string | null) => void;
        buttonClass: string;
    } = $props();

    let open = $state(false);
    let dropUp = $state(false);

    function toggle(event: MouseEvent) {
        event.stopPropagation();
        const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
        dropUp = window.innerHeight - bounds.bottom < 160;
        open = !open;
    }

    function choose(event: MouseEvent, value: string | null) {
        event.stopPropagation();
        open = false;
        if (value !== (color ?? null)) onchange(value);
    }
</script>

<svelte:window onclick={() => (open = false)} />

<div class="relative">
    <button aria-label="Folder color" title="Folder color" onclick={toggle} class={buttonClass}>
        <Icon icon="mdi:palette-outline" class="text-base" />
    </button>
    {#if open}
        <div
            class="absolute right-0 {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'} z-[100] grid w-44 grid-cols-5 gap-1.5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-2 shadow-xl"
            role="menu"
            tabindex="-1"
            onclick={(event) => event.stopPropagation()}
            onkeydown={(event) => event.key === 'Escape' && (open = false)}
        >
            {#each folderColorOptions as option (option.label)}
                {@const selected = (color ?? null) === option.value}
                <button
                    role="menuitemradio"
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
    {/if}
</div>
