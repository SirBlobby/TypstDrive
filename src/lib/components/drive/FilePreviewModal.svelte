<script lang="ts">
    import { untrack } from 'svelte';
    import { fileDataUrl } from '#lib/ts/item-info';
    import { MAX_ZOOM, MIN_ZOOM, ZOOM_STEP, previewKindFor, stepZoom } from '#lib/ts/file-preview';
    import FileViewer from '#lib/components/FileViewer.svelte';
    import Icon from '@iconify/svelte';

    let { files, startFileId, onClose }: {
        files: any[];
        startFileId: string;
        onClose: () => void;
    } = $props();

    let currentIndex = $state(untrack(() => Math.max(0, files.findIndex((file) => file.id === startFileId))));
    let zoom = $state(1);

    let currentFile = $derived(files[currentIndex]);
    let fileUrl = $derived(fileDataUrl(currentFile));
    let previewKind = $derived(previewKindFor(currentFile.mime_type, currentFile.name));
    let hasMultiple = $derived(files.length > 1);

    function showFile(index: number) {
        currentIndex = (index + files.length) % files.length;
        zoom = 1;
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onClose();
        else if (e.key === 'ArrowRight' && hasMultiple) showFile(currentIndex + 1);
        else if (e.key === 'ArrowLeft' && hasMultiple) showFile(currentIndex - 1);
        else if (previewKind === 'image' && (e.key === '+' || e.key === '=')) zoom = stepZoom(zoom, ZOOM_STEP);
        else if (previewKind === 'image' && e.key === '-') zoom = stepZoom(zoom, -ZOOM_STEP);
    }
</script>

<svelte:window onkeydown={handleKeydown} />

{#snippet toolbarButton(icon: string, label: string, onclick: () => void, disabled = false)}
    <button
        {onclick}
        {disabled}
        title={label}
        aria-label={label}
        class="p-2 rounded-md text-white/80 hover:text-white hover:bg-white/10 transition-colors disabled:opacity-30 disabled:hover:bg-transparent"
    >
        <Icon {icon} class="text-xl" />
    </button>
{/snippet}

<div class="fixed inset-0 z-[200] flex flex-col bg-black/90" role="dialog" aria-modal="true" aria-label={`Preview of ${currentFile.name}`}>
    <header class="flex items-center gap-3 px-4 py-3 text-white">
        {@render toolbarButton('mdi:close', 'Close preview', onClose)}
        <Icon icon={previewKind === 'pdf' ? 'mdi:file-pdf-box' : previewKind === 'image' ? 'mdi:image-outline' : 'mdi:file-outline'} class="text-2xl flex-shrink-0 {previewKind === 'pdf' ? 'text-red-400' : 'text-green-400'}" />
        <h2 class="flex-1 min-w-0 truncate text-sm font-medium" title={currentFile.name}>{currentFile.name}</h2>

        {#if previewKind === 'image'}
            <div class="hidden sm:flex items-center gap-1 rounded-md bg-white/10 px-1">
                {@render toolbarButton('mdi:magnify-minus-outline', 'Zoom out', () => (zoom = stepZoom(zoom, -ZOOM_STEP)), zoom <= MIN_ZOOM)}
                <button onclick={() => (zoom = 1)} title="Reset zoom" class="min-w-14 text-xs font-medium text-white/80 hover:text-white">{Math.round(zoom * 100)}%</button>
                {@render toolbarButton('mdi:magnify-plus-outline', 'Zoom in', () => (zoom = stepZoom(zoom, ZOOM_STEP)), zoom >= MAX_ZOOM)}
            </div>
        {/if}

        <a href={fileUrl} download={currentFile.name} title="Download" aria-label="Download" class="p-2 rounded-md text-white/80 hover:text-white hover:bg-white/10 transition-colors">
            <Icon icon="mdi:download" class="text-xl" />
        </a>
        <a href={fileUrl} target="_blank" rel="noopener noreferrer" title="Open in new tab" aria-label="Open in new tab" class="p-2 rounded-md text-white/80 hover:text-white hover:bg-white/10 transition-colors">
            <Icon icon="mdi:open-in-new" class="text-xl" />
        </a>
    </header>

    <div class="relative flex-1 min-h-0 flex items-center justify-center px-4 pb-4">
        {#if hasMultiple}
            <div class="absolute left-4 top-1/2 -translate-y-1/2 z-10 rounded-md bg-black/50">
                {@render toolbarButton('mdi:chevron-left', 'Previous file', () => showFile(currentIndex - 1))}
            </div>
            <div class="absolute right-4 top-1/2 -translate-y-1/2 z-10 rounded-md bg-black/50">
                {@render toolbarButton('mdi:chevron-right', 'Next file', () => showFile(currentIndex + 1))}
            </div>
        {/if}

        <div class="w-full h-full flex items-center justify-center text-white">
            {#key currentFile.id}
                <FileViewer url={fileUrl} name={currentFile.name} kind={previewKind} {zoom} />
            {/key}
        </div>
    </div>

    {#if hasMultiple}
        <p class="pb-3 text-center text-xs text-white/60">{currentIndex + 1} of {files.length}</p>
    {/if}
</div>
