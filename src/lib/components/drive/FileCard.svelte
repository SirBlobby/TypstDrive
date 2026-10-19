<script lang="ts">
    import Icon from '@iconify/svelte';

    let { file, deleteFile, openRename, openPreview } = $props<{
        file: any;
        deleteFile: (id: string, name: string) => void;
        openRename: (id: string, name: string, type: 'file') => void;
        openPreview: (file: any) => void;
    }>();

    let lowerName = $derived(file.name.toLowerCase());
    let isFont = $derived(lowerName.endsWith('.ttf') || lowerName.endsWith('.otf'));
    let isImage = $derived(file.mime_type.startsWith('image/'));
    let isPdf = $derived(file.mime_type === 'application/pdf' || lowerName.endsWith('.pdf'));
    let fileUrl = $derived(`/api/files/${file.id}/data`);
    let previewFailed = $state(false);

    let uploadedDate = $derived(
        new Date(file.created_at ? (file.created_at.endsWith('Z') ? file.created_at : file.created_at + 'Z') : Date.now())
            .toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
    );

    function handleOpen() {
        if (!isFont) {
            openPreview(file);
        }
    }
</script>

<div
    class="bg-[var(--color-surface)] rounded-lg shadow-sm border border-[var(--color-line)] flex flex-col transition relative group {isFont ? 'cursor-default' : 'hover:border-[var(--color-accent)] cursor-pointer'}"
    role="button"
    tabindex="0"
    onclick={handleOpen}
    onkeydown={(e) => e.key === 'Enter' && handleOpen()}
    draggable="true"
    ondragstart={(e) => e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'file', id: file.id }))}
>
    <div class="h-40 w-full bg-[var(--color-surface-muted)] rounded-t-lg overflow-hidden flex items-center justify-center border-b border-[var(--color-line)] pointer-events-none">
        {#if isImage && !previewFailed}
            <img
                src={fileUrl}
                alt={file.name}
                loading="lazy"
                draggable="false"
                onerror={() => (previewFailed = true)}
                class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-110"
            />
        {:else if isFont}
            <div class="p-4 bg-green-50 dark:bg-green-500/10 text-green-600 dark:text-green-400 rounded-lg transition-transform duration-300 group-hover:scale-110">
                <Icon icon="mdi:format-font" class="text-4xl" />
            </div>
        {:else}
            <div class="p-4 bg-green-50 dark:bg-green-500/10 text-green-600 dark:text-green-400 rounded-lg transition-transform duration-300 group-hover:scale-110">
                <Icon icon={isPdf ? 'mdi:file-pdf-box' : isImage ? 'mdi:image-outline' : 'mdi:file-outline'} class="text-4xl" />
            </div>
        {/if}
    </div>

    <div class="p-4 flex flex-col flex-grow">
        <div class="flex items-start justify-between gap-2">
            <h3 class="text-lg font-semibold text-[var(--color-ink)] truncate pointer-events-none" title={file.name}>{file.name}</h3>
            <div class="flex items-center gap-0.5 flex-shrink-0 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
                <button
                    aria-label="Rename file"
                    title="Rename file"
                    onclick={(e) => { e.stopPropagation(); openRename(file.id, file.name, 'file'); }}
                    class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded-md hover:bg-[var(--color-surface-sunken)]"
                >
                    <Icon icon="mdi:pencil-outline" class="text-lg" />
                </button>
                <button
                    aria-label="Delete file"
                    title="Delete file"
                    onclick={(e) => { e.stopPropagation(); deleteFile(file.id, file.name); }}
                    class="text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] p-1 rounded-md hover:bg-[var(--color-danger)]/10"
                >
                    <Icon icon="mdi:trash-can-outline" class="text-lg" />
                </button>
            </div>
        </div>
        <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-1 mt-2 pointer-events-none">
            <Icon icon={isFont ? 'mdi:format-font' : isImage ? 'mdi:image-outline' : 'mdi:clock-outline'} class="text-sm" />
            Uploaded {uploadedDate}
        </p>
    </div>
</div>
