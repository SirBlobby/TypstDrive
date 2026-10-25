<script lang="ts">
    import { page } from '$app/state';
    import LinkGate from '#lib/components/sharing/LinkGate.svelte';

    let { children } = $props();
</script>

{#key page.params.id}
    <LinkGate kind="document" itemId={page.params.id ?? ''}>
        {@render children()}
    </LinkGate>
{/key}
