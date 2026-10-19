<script lang="ts">
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import UserAvatar from '$lib/components/UserAvatar.svelte';

    const maxUploadBytes = 15 * 1024 * 1024;

    let fileInput: HTMLInputElement;
    let isUploading = $state(false);
    let uploadError = $state('');

    function setAvatarVersion(version: string | null) {
        userStore.update((user) => (user ? { ...user, avatar_updated_at: version } : user));
    }

    async function uploadAvatar(event: Event) {
        const input = event.target as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;

        uploadError = '';
        if (file.size > maxUploadBytes) {
            uploadError = 'Image must be 15 MB or smaller.';
            return;
        }

        isUploading = true;
        try {
            const formData = new FormData();
            formData.append('avatar', file);
            const res = await fetch('/api/auth/avatar', { method: 'PUT', body: formData });
            if (res.ok) {
                setAvatarVersion(await res.text());
            } else {
                uploadError = (await res.text()) || 'Failed to upload image.';
            }
        } catch {
            uploadError = 'Network error occurred.';
        }
        isUploading = false;
    }

    async function removeAvatar() {
        uploadError = '';
        isUploading = true;
        try {
            const res = await fetch('/api/auth/avatar', { method: 'DELETE' });
            if (res.ok) {
                setAvatarVersion(null);
            } else {
                uploadError = (await res.text()) || 'Failed to remove image.';
            }
        } catch {
            uploadError = 'Network error occurred.';
        }
        isUploading = false;
    }
</script>

<div class="flex flex-col items-center gap-2 flex-shrink-0">
    <button
        type="button"
        onclick={() => fileInput.click()}
        disabled={isUploading}
        class="relative group rounded-full disabled:cursor-wait"
        title="Change profile picture"
    >
        <UserAvatar sizeClass="h-20 w-20" textClass="text-3xl" />
        <span class="absolute inset-0 rounded-full bg-black/50 flex items-center justify-center text-white opacity-0 group-hover:opacity-100 transition-opacity {isUploading ? 'opacity-100' : ''}">
            <Icon icon={isUploading ? 'mdi:loading' : 'mdi:camera-outline'} class="text-2xl {isUploading ? 'animate-spin' : ''}" />
        </span>
    </button>
    <input bind:this={fileInput} type="file" accept="image/png,image/jpeg,image/gif,image/webp" class="hidden" onchange={uploadAvatar} />
    {#if $userStore?.avatar_updated_at}
        <button type="button" onclick={removeAvatar} disabled={isUploading} class="text-xs text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] transition-colors disabled:opacity-50">
            Remove
        </button>
    {/if}
    {#if uploadError}
        <p class="text-xs text-[var(--color-danger)] max-w-40 text-center">{uploadError}</p>
    {/if}
</div>
