<script lang="ts">
    import { userStore, avatarUrl } from '#lib/ts/auth';

    let { sizeClass = 'h-9 w-9', textClass = 'text-base' }: { sizeClass?: string; textClass?: string } = $props();

    let imageSource = $derived(avatarUrl($userStore));
    let initial = $derived($userStore?.username?.charAt(0).toUpperCase() || '?');
</script>

{#if imageSource}
    <img src={imageSource} alt="Profile" class="{sizeClass} rounded-full object-cover flex-shrink-0" />
{:else}
    <div class="{sizeClass} {textClass} rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] font-bold flex-shrink-0">
        {initial}
    </div>
{/if}
