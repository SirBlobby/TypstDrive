<script lang="ts">
	import { connectedUsers, type AwarenessUser } from '$lib/ts/store';

	let failedImages = $state<Set<string>>(new Set());

	function initials(name: string) {
		return name.substring(0, 2).toUpperCase();
	}

	function showImage(user: AwarenessUser) {
		return !!user.avatar && !failedImages.has(user.avatar);
	}

	function markFailed(source: string) {
		failedImages = new Set([...failedImages, source]);
	}
</script>

{#if $connectedUsers.length > 0}
	<div class="flex items-center -space-x-2 mr-2">
		{#each $connectedUsers as user (user.clientId)}
			<div
				class="w-7 h-7 rounded-full overflow-hidden flex items-center justify-center text-xs font-bold text-white border-2 shadow-sm"
				style="background-color: {user.color}; border-color: {user.color}; z-index: {user.isLocal ? 10 : 1};"
				title={user.name + (user.isLocal ? ' (You)' : '')}
			>
				{#if showImage(user)}
					<img src={user.avatar} alt={user.name} class="h-full w-full object-cover" onerror={() => user.avatar && markFailed(user.avatar)} />
				{:else}
					{initials(user.name)}
				{/if}
			</div>
		{/each}
	</div>
{/if}
