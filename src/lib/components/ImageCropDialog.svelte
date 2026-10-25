<script lang="ts">
    import Cropper from 'svelte-easy-crop';
    import Icon from '@iconify/svelte';
    import { cropImage, type CropArea } from '#lib/ts/image-crop';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';

    const MAX_ZOOM = 4;

    let { image, title, round = false, outputSize = 512, onCancel, onConfirm }: {
        image: string;
        title: string;
        round?: boolean;
        outputSize?: number;
        onCancel: () => void;
        onConfirm: (cropped: Blob) => Promise<void>;
    } = $props();

    let crop = $state({ x: 0, y: 0 });
    let zoom = $state(1);
    let croppedArea = $state<CropArea | null>(null);
    let working = $state(false);
    let errorMsg = $state('');

    async function confirm() {
        if (!croppedArea) return;
        working = true;
        errorMsg = '';
        try {
            await onConfirm(await cropImage(image, croppedArea, outputSize));
        } catch (err) {
            errorMsg = (err as Error).message;
        }
        working = false;
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape' && !working) onCancel();
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true" aria-label={title}>
    <div class="w-full max-w-md overflow-hidden rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] shadow-2xl">
        <div class="flex items-center justify-between border-b border-[var(--color-line)] px-5 py-3">
            <h2 class="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
            <button type="button" onclick={onCancel} disabled={working} aria-label="Close" class="rounded-md p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]">
                <Icon icon="mdi:close" class="text-lg" />
            </button>
        </div>

        <div class="relative h-72 w-full bg-black">
            <Cropper
                {image}
                bind:crop
                bind:zoom
                aspect={1}
                maxZoom={MAX_ZOOM}
                cropShape={round ? 'round' : 'rect'}
                showGrid={false}
                oncropcomplete={(event) => (croppedArea = event.pixels)}
            />
        </div>

        <div class="space-y-4 px-5 py-4">
            <label class="flex items-center gap-3 text-[var(--color-ink-muted)]">
                <Icon icon="mdi:magnify-minus-outline" class="text-lg" />
                <input type="range" min="1" max={MAX_ZOOM} step="0.01" bind:value={zoom} aria-label="Zoom" class="flex-1 accent-[var(--color-accent)]" />
                <Icon icon="mdi:magnify-plus-outline" class="text-lg" />
            </label>
            {#if errorMsg}
                <p class="text-xs text-[var(--color-danger)]">{errorMsg}</p>
            {/if}
            <div class="flex justify-end gap-2">
                <button type="button" onclick={onCancel} disabled={working} class={secondaryButtonClass}>Cancel</button>
                <button type="button" onclick={confirm} disabled={working || !croppedArea} class={primaryButtonClass}>
                    <Icon icon={working ? 'mdi:loading' : 'mdi:check'} class="text-base {working ? 'animate-spin' : ''}" />
                    {working ? 'Saving...' : 'Save'}
                </button>
            </div>
        </div>
    </div>
</div>
