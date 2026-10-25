<script lang="ts">
	import type { AwarenessUser } from '#lib/ts/store';

	let { user, sizeClass }: { user: AwarenessUser; sizeClass: string } = $props();

	let failed = $state(false);
</script>

<span
	class="flex items-center justify-center overflow-hidden rounded-full border-2 font-bold text-white shadow-sm {sizeClass}"
	style="background-color: {user.color}; border-color: {user.color};"
>
	{#if user.avatar && !failed}
		<img src={user.avatar} alt={user.name} class="h-full w-full object-cover" onerror={() => (failed = true)} />
	{:else}
		{user.name.substring(0, 2).toUpperCase()}
	{/if}
</span>
