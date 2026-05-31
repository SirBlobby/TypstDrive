<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';
	import Navbar from '$lib/components/dashboard/Navbar.svelte';

	interface Package {
		id: string;
		name: string;
		description?: string;
		owner_name?: string;
		latest_version?: string;
	}

	let packages = $state<Package[]>([]);
	let loading = $state(true);
	let copied = $state('');

	async function load() {
		loading = true;
		const res = await fetch('/api/packages');
		packages = res.ok ? await res.json() : [];
		loading = false;
	}

	function importSnippet(pkg: Package): string {
		return `#import "@typstdrive/${pkg.name}:${pkg.latest_version ?? '0.1.0'}": *`;
	}

	async function copy(pkg: Package) {
		await navigator.clipboard.writeText(importSnippet(pkg));
		copied = pkg.id;
		setTimeout(() => (copied = ''), 2000);
	}

	async function remove(pkg: Package) {
		if (!confirm(`Delete package "${pkg.name}" and all its versions?`)) return;
		const res = await fetch(`/api/packages/${pkg.name}`, { method: 'DELETE' });
		if (res.ok) packages = packages.filter((p) => p.id !== pkg.id);
	}

	onMount(load);
</script>

<svelte:head>
	<title>Packages - TypstDrive</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-[var(--theme-bg)]">
	<Navbar />

	<div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
		<div class="mb-6">
			<button onclick={() => goto('/dashboard')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors mb-2 flex items-center gap-1.5">
				<Icon icon="mdi:arrow-left" class="text-lg" />
				Back to Dashboard
			</button>
			<h2 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
				<Icon icon="mdi:package-variant-closed" class="text-purple-500" />
				Packages
			</h2>
			<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
				Instance-local Typst packages, published from Spaces and importable as
				<code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">@typstdrive/&lt;name&gt;:&lt;version&gt;</code>.
			</p>
		</div>

		{#if loading}
			<p class="text-gray-500 dark:text-gray-400">Loading…</p>
		{:else if packages.length === 0}
			<div class="text-center py-16 text-gray-500 dark:text-gray-400">
				<Icon icon="mdi:package-variant" class="text-5xl mx-auto mb-3 opacity-50" />
				<p>No packages published yet. Open a Space and use “Publish” to create one.</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each packages as pkg (pkg.id)}
					<div class="bg-white dark:bg-black/20 rounded-xl border border-gray-200 dark:border-white/10 p-4 flex items-start justify-between gap-4">
						<div class="min-w-0">
							<div class="flex items-center gap-2">
								<p class="font-semibold text-gray-900 dark:text-white truncate">@typstdrive/{pkg.name}</p>
								{#if pkg.latest_version}
									<span class="text-xs font-mono bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 px-1.5 py-0.5 rounded">v{pkg.latest_version}</span>
								{/if}
							</div>
							{#if pkg.description}
								<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 truncate">{pkg.description}</p>
							{/if}
							<p class="text-xs text-gray-400 mt-1">by {pkg.owner_name ?? 'unknown'}</p>
							<pre class="mt-2 text-xs font-mono bg-gray-50 dark:bg-black/30 border border-gray-100 dark:border-white/5 rounded px-2 py-1 overflow-x-auto">{importSnippet(pkg)}</pre>
						</div>
						<div class="flex flex-col items-end gap-2 flex-shrink-0">
							<button onclick={() => copy(pkg)} class="text-xs px-2 py-1 rounded-md bg-gray-100 dark:bg-white/5 hover:bg-gray-200 dark:hover:bg-white/10 flex items-center gap-1">
								<Icon icon={copied === pkg.id ? 'mdi:check' : 'mdi:content-copy'} class="text-sm" />
								{copied === pkg.id ? 'Copied' : 'Copy'}
							</button>
							<button onclick={() => remove(pkg)} title="Delete" class="text-xs px-2 py-1 rounded-md text-gray-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center gap-1">
								<Icon icon="mdi:trash-can-outline" class="text-sm" /> Delete
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
