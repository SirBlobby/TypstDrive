<script lang="ts">
	import Icon from '@iconify/svelte';

	let {
		spaceId,
		onClose
	}: {
		spaceId: string;
		onClose: () => void;
	} = $props();

	let version = $state('');
	let publishing = $state(false);
	let error = $state('');
	let success = $state('');

	async function publish() {
		publishing = true;
		error = '';
		success = '';
		try {
			const res = await fetch('/api/packages/publish', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ space_id: spaceId, version: version.trim() || undefined })
			});
			if (!res.ok) {
				error = await res.text();
			} else {
				const pkg = await res.json();
				success = `Published @typstdrive/${pkg.name}`;
			}
		} catch (e) {
			error = 'Network error while publishing.';
		}
		publishing = false;
	}
</script>

<div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50" onclick={onClose} role="presentation">
	<div class="bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 w-full max-w-md p-6" onclick={(e) => e.stopPropagation()} role="presentation">
		<div class="flex items-center gap-2 mb-4">
			<Icon icon="mdi:package-variant-closed" class="text-2xl text-purple-500" />
			<h2 class="text-lg font-bold">Publish as Package</h2>
		</div>

		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Snapshots this space's files into an immutable package version, importable instance-wide as
			<code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">@typstdrive/&lt;name&gt;:&lt;version&gt;</code>.
			The name, version and entrypoint come from your <code class="font-mono text-xs bg-gray-100 dark:bg-white/10 px-1.5 py-0.5 rounded">typst.toml</code>.
		</p>

		<label class="block text-sm font-medium mb-1" for="pkg-version">Version override (optional)</label>
		<input
			id="pkg-version"
			bind:value={version}
			placeholder="e.g. 0.1.0 (defaults to typst.toml)"
			class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-white/10 bg-transparent text-sm mb-4 focus:outline-none focus:ring-2 focus:ring-purple-500/40"
		/>

		{#if error}
			<div class="text-sm text-red-600 dark:text-red-400 mb-3 break-words">{error}</div>
		{/if}
		{#if success}
			<div class="text-sm text-green-600 dark:text-green-400 mb-3">{success}</div>
		{/if}

		<div class="flex justify-end gap-2">
			<button onclick={onClose} class="px-4 py-2 text-sm rounded-lg bg-gray-100 dark:bg-white/5 hover:bg-gray-200 dark:hover:bg-white/10">Close</button>
			<button onclick={publish} disabled={publishing} class="px-4 py-2 text-sm rounded-lg bg-purple-600 text-white hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2">
				{#if publishing}<Icon icon="mdi:loading" class="animate-spin" />{/if}
				Publish
			</button>
		</div>
	</div>
</div>
