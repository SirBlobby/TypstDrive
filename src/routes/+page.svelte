<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { userStore } from '$lib/ts/auth';

    onMount(() => {
        if ($userStore) {
            goto('/dashboard');
        } else {
            goto('/login');
        }
    });
</script>

<svelte:head>
	<title>TypstDrive</title>
	<meta name="description" content="Collaborative Typst Editor." />
</svelte:head>

<div class="h-full flex items-center justify-center text-gray-500">
    Redirecting...
</div>
