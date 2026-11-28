<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { connectedUsers, type AwarenessUser } from '#lib/ts/store';
	import { followingStore, stopFollowing, toggleFollow } from '#lib/ts/follow';
	import UserAvatar from './UserAvatar.svelte';
	import EditingModeBadge, { editingModeDetails } from './EditingModeBadge.svelte';

	let followedUser = $derived($connectedUsers.find((user) => !user.isLocal && user.session && user.session === $followingStore) ?? null);

	function canFollow(user: AwarenessUser) {
		return !user.isLocal && !!user.session;
	}

	function modeSuffix(user: AwarenessUser) {
		return user.mode ? ` · ${editingModeDetails[user.mode].label}` : '';
	}

	function label(user: AwarenessUser) {
		if (user.isLocal) return `${user.name} (You)${modeSuffix(user)}`;
		if (!user.session) return `${user.name}${modeSuffix(user)}`;
		const action = user.session === $followingStore ? `Stop following ${user.name}` : `Follow ${user.name}`;
		return `${action}${modeSuffix(user)}`;
	}
</script>

{#if $connectedUsers.length > 0}
	<div class="flex items-center gap-2 mr-2">
		{#if followedUser}
			<button
				onclick={stopFollowing}
				class="flex items-center gap-1 rounded-full py-0.5 pl-2 pr-1.5 text-xs font-medium text-white"
				style="background-color: {followedUser.color};"
				title="Stop following"
			>
				Following {followedUser.name}
				<Icon icon="mdi:close" class="text-sm" />
			</button>
		{/if}
		<div class="flex items-center -space-x-2">
			{#each $connectedUsers as user (user.clientId)}
				{#if canFollow(user)}
					<button
						onclick={() => user.session && toggleFollow(user.session)}
						title={label(user)}
						aria-label={label(user)}
						aria-pressed={user.session === $followingStore}
						class="relative rounded-full transition-transform hover:z-20 hover:-translate-y-0.5 {user.session === $followingStore ? 'z-20 ring-2 ring-offset-2 ring-offset-[var(--color-surface)]' : 'z-[1]'}"
						style="--tw-ring-color: {user.color};"
					>
						<UserAvatar {user} sizeClass="w-7 h-7 text-xs" />
						{#if user.mode}
							<EditingModeBadge mode={user.mode} />
						{/if}
					</button>
				{:else}
					<span title={label(user)} class="relative {user.isLocal ? 'z-10' : 'z-[1]'}">
						<UserAvatar {user} sizeClass="w-7 h-7 text-xs" />
						{#if user.mode}
							<EditingModeBadge mode={user.mode} />
						{/if}
					</span>
				{/if}
			{/each}
		</div>
	</div>
{/if}
