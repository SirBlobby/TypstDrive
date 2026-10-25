<script lang="ts">
    import Icon from '@iconify/svelte';
    import { replaceFileData } from '#lib/ts/item-info';

    let { file, onReplaced, buttonClass, iconClass = 'text-base' }: {
        file: { id: string; name: string };
        onReplaced: (updated: any) => void;
        buttonClass: string;
        iconClass?: string;
    } = $props();

    let fileInput: HTMLInputElement;
    let busy = $state(false);

    async function replace(event: Event) {
        const input = event.target as HTMLInputElement;
        const replacement = input.files?.[0];
        input.value = '';
        if (!replacement) return;

        busy = true;
        try {
            onReplaced(await replaceFileData(file.id, replacement));
        } catch (err) {
            alert((err as Error).message);
        }
        busy = false;
    }
</script>

<button
    type="button"
    aria-label={`Replace ${file.name}`}
    title="Replace with a new upload"
    disabled={busy}
    onclick={(e) => { e.stopPropagation(); fileInput.click(); }}
    onkeydown={(e) => e.stopPropagation()}
    class={buttonClass}
>
    <Icon icon={busy ? 'mdi:loading' : 'mdi:file-replace-outline'} class="{iconClass} {busy ? 'animate-spin' : ''}" />
</button>
<input bind:this={fileInput} type="file" class="hidden" onclick={(e) => e.stopPropagation()} onchange={replace} />
