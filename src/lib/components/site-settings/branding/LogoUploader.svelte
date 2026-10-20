<script lang="ts">
    import Icon from '@iconify/svelte';
    import { brandingStore, logoUrl } from '$lib/ts/branding';
    import { secondaryButtonClass } from '$lib/ts/settings-ui';
    import ImageCropDialog from '$lib/components/ImageCropDialog.svelte';

    const MAX_UPLOAD_BYTES = 15 * 1024 * 1024;
    const LOGO_OUTPUT_SIZE = 256;

    let fileInput: HTMLInputElement;
    let cropSource = $state<string | null>(null);
    let busy = $state(false);
    let errorMsg = $state('');

    function chooseImage(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;

        errorMsg = '';
        if (file.size > MAX_UPLOAD_BYTES) {
            errorMsg = 'Image must be 15 MB or smaller.';
            return;
        }
        cropSource = URL.createObjectURL(file);
    }

    function closeCropper() {
        if (cropSource) URL.revokeObjectURL(cropSource);
        cropSource = null;
    }

    async function uploadLogo(image: Blob) {
        busy = true;
        try {
            const formData = new FormData();
            formData.append('logo', image, 'logo.png');
            const res = await fetch('/api/admin/branding/logo', { method: 'PUT', body: formData });
            if (!res.ok) throw new Error((await res.text()) || 'Failed to upload logo.');
            brandingStore.set(await res.json());
            closeCropper();
        } finally {
            busy = false;
        }
    }

    async function removeLogo() {
        busy = true;
        errorMsg = '';
        try {
            const res = await fetch('/api/admin/branding/logo', { method: 'DELETE' });
            if (res.ok) brandingStore.set(await res.json());
            else errorMsg = (await res.text()) || 'Failed to remove logo.';
        } catch {
            errorMsg = 'Network error.';
        }
        busy = false;
    }
</script>

<div class="flex flex-wrap items-center justify-end gap-3">
    <span class="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-[var(--color-accent)]">
        <img src={logoUrl($brandingStore)} alt="Current logo" class="h-8 w-8 object-contain" />
    </span>
    <div class="flex flex-wrap items-center gap-2">
        <button type="button" onclick={() => fileInput.click()} disabled={busy} class={secondaryButtonClass}>
            <Icon icon="mdi:upload" class="text-base" />
            Upload logo
        </button>
        {#if $brandingStore.logo_updated_at}
            <button type="button" onclick={removeLogo} disabled={busy} class={secondaryButtonClass}>
                <Icon icon="mdi:restore" class="text-base" />
                Use default logo
            </button>
        {/if}
    </div>
    <input bind:this={fileInput} type="file" accept="image/png,image/jpeg,image/gif,image/webp" class="hidden" onchange={chooseImage} />
    {#if errorMsg}
        <p class="w-full text-right text-xs text-[var(--color-danger)]">{errorMsg}</p>
    {/if}
</div>

{#if cropSource}
    <ImageCropDialog image={cropSource} title="Crop logo" outputSize={LOGO_OUTPUT_SIZE} onCancel={closeCropper} onConfirm={uploadLogo} />
{/if}
