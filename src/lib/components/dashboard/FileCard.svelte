<script lang="ts">
    import Icon from '@iconify/svelte';

    let { file, deleteFile } = $props<{
        file: any;
        deleteFile: (id: string, name: string) => void;
    }>();

    let isFont = $derived(file.name.endsWith('.ttf') || file.name.endsWith('.otf'));

    function handleOpen() {
        if (!isFont) {
            window.open(`/api/files/${file.id}/data`, '_blank');
        }
    }
</script>

<div
    class="bg-[var(--color-surface)] rounded-lg shadow-sm border border-[var(--color-line)] p-6 flex flex-col transition relative group {isFont ? 'cursor-default' : 'hover:border-[var(--color-accent)] cursor-pointer'}"
    role="button"
    tabindex="0"
    onclick={handleOpen}
    onkeydown={(e) => e.key === 'Enter' && handleOpen()}
    draggable="true"
    ondragstart={(e) => e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'file', id: file.id }))}
>
    <div class="flex items-start justify-between mb-4 pointer-events-none">
        <div class="p-3 bg-green-50 dark:bg-green-500/10 text-green-600 dark:text-green-400 rounded-md overflow-hidden flex items-center justify-center w-12 h-12">
            {#if file.mime_type.startsWith('image/')}
                <img src={`/api/files/${file.id}/data`} alt={file.name} class="w-full h-full object-cover rounded" draggable="false" />
            {:else if isFont}
                <Icon icon="mdi:format-font" class="text-2xl" />
            {:else}
                <Icon icon="mdi:file-outline" class="text-2xl" />
            {/if}
        </div>
        <button aria-label="Delete file" onclick={(e) => { e.stopPropagation(); deleteFile(file.id, file.name); }} class="pointer-events-auto text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] opacity-0 group-hover:opacity-100 transition-opacity bg-[var(--color-surface-muted)] hover:bg-[var(--color-danger)]/10 rounded-full p-2 shadow-sm border border-[var(--color-line)]">
            <Icon icon="mdi:trash-can-outline" class="text-lg" />
        </button>
    </div>
    <h3 class="text-lg font-semibold text-[var(--color-ink)] truncate mb-1 pointer-events-none" title={file.name}>{file.name}</h3>
    <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-1 mt-auto pt-4 border-t border-[var(--color-line)] pointer-events-none">
        <Icon icon="mdi:clock-outline" class="text-sm" />
        Uploaded {new Date(file.created_at ? (file.created_at.endsWith('Z') ? file.created_at : file.created_at + 'Z') : Date.now()).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
    </p>
</div>
