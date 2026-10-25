<script lang="ts">
    import { brandingStore } from '#lib/ts/branding';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { userStore } from '#lib/ts/auth';

    onMount(() => {
        if ($userStore) {
            goto('/drive');
        } else {
            goto('/login');
        }
    });
</script>

<svelte:head>
	<title>{$brandingStore.site_name}</title>
	<meta name="description" content="Collaborative Typst Editor." />
</svelte:head>

<div class="h-full flex items-center justify-center text-[var(--color-ink-muted)]">
    Redirecting...
</div>
