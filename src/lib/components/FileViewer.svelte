<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import type { PreviewKind } from '#lib/ts/drive/file-preview';

    let { url, name, kind, zoom = 1, pdfClass = 'max-w-5xl rounded-lg' }: {
        url: string;
        name: string;
        kind: PreviewKind;
        zoom?: number;
        pdfClass?: string;
    } = $props();

    let imageFailed = $state(false);
</script>

{#if kind === 'image' && !imageFailed}
    <div class="w-full h-full overflow-auto flex items-center justify-center">
        <img
            src={url}
            alt={name}
            onerror={() => (imageFailed = true)}
            style="transform: scale({zoom}); transform-origin: center;"
            class="max-w-full max-h-full object-contain transition-transform duration-150 select-none"
            draggable="false"
        />
    </div>
{:else if kind === 'pdf'}
    <iframe src={url} title={name} class="w-full h-full bg-white {pdfClass}"></iframe>
{:else}
    <div class="flex flex-col items-center gap-4 text-center opacity-80">
        <Icon icon="mdi:file-eye-outline" class="text-6xl" />
        <p class="text-sm">{imageFailed ? 'This image could not be loaded.' : 'No preview available for this file.'}</p>
        <a href={url} download={name} class="inline-flex items-center gap-2 rounded-md border border-current px-4 py-2 text-sm hover:opacity-80 transition-opacity">
            <Icon icon="mdi:download" class="text-lg" />
            Download
        </a>
    </div>
{/if}
