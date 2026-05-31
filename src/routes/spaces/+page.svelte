<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';
	import Navbar from '$lib/components/dashboard/Navbar.svelte';
	import SpaceCard from '$lib/components/dashboard/SpaceCard.svelte';

	interface Space {
		id: string;
		name: string;
		entrypoint: string;
		thumbnail_svg?: string;
		updated_at: string;
		effective_role?: string;
	}

	let spaces = $state<Space[]>([]);
	let shared = $state<Space[]>([]);
	let loading = $state(true);
	let showCreate = $state(false);
	let newName = $state('');
	let creating = $state(false);

	let activeMenu = $state<string | null>(null);
	let showRename = $state(false);
	let renameId = $state('');
	let renameName = $state('');
	let showInfo = $state(false);
	let infoSpace = $state<Space | null>(null);

	function setActiveMenu(id: string | null) { activeMenu = id; }
	function openInfo(space: Space) { activeMenu = null; infoSpace = space; showInfo = true; }
	function openRename(id: string, name: string) { activeMenu = null; renameId = id; renameName = name; showRename = true; }

	async function submitRename(e: Event) {
		e.preventDefault();
		if (!renameName.trim()) return;
		const res = await fetch(`/api/spaces/${renameId}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name: renameName.trim() })
		});
		if (res.ok) {
			spaces = spaces.map((s) => (s.id === renameId ? { ...s, name: renameName.trim() } : s));
		}
		showRename = false;
	}

	function handleWindowClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.action-menu-container')) activeMenu = null;
	}

	async function load() {
		loading = true;
		const [own, sh] = await Promise.all([
			fetch('/api/spaces').then((r) => (r.ok ? r.json() : [])),
			fetch('/api/spaces/shared').then((r) => (r.ok ? r.json() : []))
		]);
		spaces = own;
		shared = sh;
		loading = false;
	}

	async function create() {
		if (!newName.trim()) return;
		creating = true;
		const res = await fetch('/api/spaces', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name: newName.trim() })
		});
		creating = false;
		if (res.ok) {
			const space = await res.json();
			goto(`/space/${space.id}`);
		}
	}

	async function remove(id: string, name: string) {
		if (!confirm(`Delete space "${name}"? This cannot be undone.`)) return;
		const res = await fetch(`/api/spaces/${id}`, { method: 'DELETE' });
		if (res.ok) spaces = spaces.filter((s) => s.id !== id);
	}

	onMount(load);
</script>

<svelte:head>
	<title>Spaces - TypstDrive</title>
</svelte:head>

<svelte:window onclick={handleWindowClick} />

<div class="min-h-screen bg-gray-50 dark:bg-[var(--theme-bg)]">
	<Navbar />

	<div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
		<div class="flex items-center justify-between mb-6">
			<div>
				<button onclick={() => goto('/dashboard')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors mb-2 flex items-center gap-1.5">
					<Icon icon="mdi:arrow-left" class="text-lg" />
					Back to Dashboard
				</button>
				<h2 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
					<Icon icon="mdi:folder-multiple-outline" class="text-blue-500" />
					Spaces
				</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">Multi-file Typst workspaces with their own <code class="font-mono text-xs">typst.toml</code>.</p>
			</div>
			<button onclick={() => { showCreate = true; newName = ''; }} class="px-4 py-2 text-sm rounded-lg bg-blue-600 text-white hover:bg-blue-700 flex items-center gap-2">
				<Icon icon="mdi:plus" class="text-lg" /> New Space
			</button>
		</div>

		{#if loading}
			<p class="text-gray-500 dark:text-gray-400">Loading…</p>
		{:else}
			{#if spaces.length === 0}
				<div class="text-center py-16 text-gray-500 dark:text-gray-400">
					<Icon icon="mdi:folder-multiple-outline" class="text-5xl mx-auto mb-3 opacity-50" />
					<p>No spaces yet. Create one to start a multi-file project.</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each spaces as space (space.id)}
						<SpaceCard
							{space}
							{activeMenu}
							{setActiveMenu}
							{openInfo}
							{openRename}
							deleteSpace={remove}
						/>
					{/each}
				</div>
			{/if}

			{#if shared.length > 0}
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white mt-10 mb-4 flex items-center gap-2">
					<Icon icon="mdi:account-group-outline" class="text-blue-500" /> Shared with me
				</h3>
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each shared as space (space.id)}
						<button onclick={() => goto(`/space/${space.id}`)} class="text-left bg-white dark:bg-black/20 rounded-xl border border-gray-200 dark:border-white/10 overflow-hidden hover:shadow-md transition-shadow">
							<div class="h-32 bg-gray-50 dark:bg-black/30 flex items-center justify-center overflow-hidden border-b border-gray-100 dark:border-white/5">
								{#if space.thumbnail_svg}
									{@html space.thumbnail_svg}
								{:else}
									<Icon icon="mdi:folder-multiple-outline" class="text-4xl text-gray-300 dark:text-gray-600" />
								{/if}
							</div>
							<div class="p-3">
								<p class="font-medium text-gray-900 dark:text-white truncate">{space.name}</p>
								<p class="text-xs text-gray-400 mt-0.5">{space.effective_role}</p>
							</div>
						</button>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

{#if showCreate}
	<div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50" onclick={() => (showCreate = false)} role="presentation">
		<div class="bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 w-full max-w-md p-6" onclick={(e) => e.stopPropagation()} role="presentation">
			<h2 class="text-lg font-bold mb-4 flex items-center gap-2"><Icon icon="mdi:folder-plus-outline" class="text-blue-500" /> New Space</h2>
			<input bind:value={newName} placeholder="Space name" onkeydown={(e) => e.key === 'Enter' && create()} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-white/10 bg-transparent text-sm mb-4 focus:outline-none focus:ring-2 focus:ring-blue-500/40" />
			<div class="flex justify-end gap-2">
				<button onclick={() => (showCreate = false)} class="px-4 py-2 text-sm rounded-lg bg-gray-100 dark:bg-white/5 hover:bg-gray-200 dark:hover:bg-white/10">Cancel</button>
				<button onclick={create} disabled={creating} class="px-4 py-2 text-sm rounded-lg bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50">Create</button>
			</div>
		</div>
	</div>
{/if}

{#if showRename}
	<div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50" onclick={() => (showRename = false)} role="presentation">
		<form onsubmit={submitRename} class="bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 w-full max-w-md p-6" onclick={(e) => e.stopPropagation()}>
			<h2 class="text-lg font-bold mb-4 flex items-center gap-2"><Icon icon="mdi:pencil-outline" class="text-yellow-500" /> Rename Space</h2>
			<input bind:value={renameName} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-white/10 bg-transparent text-sm mb-4 focus:outline-none focus:ring-2 focus:ring-blue-500/40" />
			<div class="flex justify-end gap-2">
				<button type="button" onclick={() => (showRename = false)} class="px-4 py-2 text-sm rounded-lg bg-gray-100 dark:bg-white/5 hover:bg-gray-200 dark:hover:bg-white/10">Cancel</button>
				<button type="submit" class="px-4 py-2 text-sm rounded-lg bg-blue-600 text-white hover:bg-blue-700">Save</button>
			</div>
		</form>
	</div>
{/if}

{#if showInfo && infoSpace}
	<div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50" onclick={() => (showInfo = false)} role="presentation">
		<div class="bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 w-full max-w-sm overflow-hidden" onclick={(e) => e.stopPropagation()} role="presentation">
			<div class="p-6 border-b border-gray-100 dark:border-white/10 flex items-center gap-3">
				<Icon icon="mdi:folder-multiple-outline" class="text-xl text-blue-500" />
				<h3 class="text-lg font-semibold flex-grow truncate">{infoSpace.name}</h3>
			</div>
			<div class="p-6 space-y-4 text-sm">
				<div><p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Entrypoint</p><p class="font-mono">{infoSpace.entrypoint}</p></div>
				<div><p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Last Modified</p><p>{new Date(infoSpace.updated_at.endsWith('Z') ? infoSpace.updated_at : infoSpace.updated_at + 'Z').toLocaleString()}</p></div>
			</div>
			<div class="p-4 bg-gray-50 dark:bg-white/5 border-t border-gray-100 dark:border-white/10 flex justify-end">
				<button onclick={() => (showInfo = false)} class="px-5 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg">Close</button>
			</div>
		</div>
	</div>
{/if}
