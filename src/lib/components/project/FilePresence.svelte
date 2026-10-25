<script lang="ts">
	import type { AwarenessUser } from '#lib/ts/store';
	import UserAvatar from '../editor/UserAvatar.svelte';

	const MAX_SHOWN = 3;

	let { users }: { users: AwarenessUser[] } = $props();

	let shown = $derived(users.slice(0, MAX_SHOWN));
	let names = $derived(users.map((user) => user.name).join(', '));
</script>

<span class="flex flex-shrink-0 items-center -space-x-1.5" title="Here now: {names}" aria-label="Here now: {names}">
	{#each shown as user (user.session ?? user.clientId)}
		<UserAvatar {user} sizeClass="h-4 w-4 border text-[7px]" />
	{/each}
	{#if users.length > MAX_SHOWN}
		<span class="flex h-4 min-w-4 items-center justify-center rounded-full bg-[var(--color-surface-sunken)] px-1 text-[8px] font-semibold text-[var(--color-ink-muted)]">+{users.length - MAX_SHOWN}</span>
	{/if}
</span>
