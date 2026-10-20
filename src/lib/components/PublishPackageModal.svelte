<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import Modal from './Modal.svelte';
	import { fetchUserPackagePolicy, type UserPackagePolicy } from '$lib/ts/packages';
	import { formatBytes } from '$lib/ts/settings-ui';

	let {
		projectId,
		onClose
	}: {
		projectId: string;
		onClose: () => void;
	} = $props();

	let version = $state('');
	let publishing = $state(false);
	let error = $state('');
	let success = $state('');
	let policy = $state<UserPackagePolicy | null>(null);

	let blockedReason = $derived.by(() => {
		if (!policy || policy.can_publish) return '';
		return policy.publishing_enabled
			? 'Only administrators can publish packages on this instance.'
			: 'Package publishing is disabled on this instance.';
	});
	let limitsLabel = $derived.by(() => {
		if (!policy) return '';
		const limits = [];
		if (policy.max_package_size_bytes > 0) limits.push(`up to ${formatBytes(policy.max_package_size_bytes)}`);
		if (policy.max_versions_per_package > 0) limits.push(`at most ${policy.max_versions_per_package} versions per package`);
		return limits.length > 0 ? `Limits: ${limits.join(', ')}.` : '';
	});

	onMount(async () => {
		policy = await fetchUserPackagePolicy();
	});

	async function publish() {
		publishing = true;
		error = '';
		success = '';
		try {
			const res = await fetch('/api/packages/publish', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ project_id: projectId, version: version.trim() || undefined })
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

<Modal title="Publish as package" icon="ph:package" onclose={onClose}>
	<p class="text-sm text-[var(--color-ink-muted)] mb-4">
		Snapshots this project's files into an immutable package version, importable instance-wide as
		<code class="font-mono text-xs bg-[var(--color-surface-sunken)] px-1.5 py-0.5 rounded">@typstdrive/&lt;name&gt;:&lt;version&gt;</code>.
		The name, version and entrypoint come from your <code class="font-mono text-xs bg-[var(--color-surface-sunken)] px-1.5 py-0.5 rounded">typst.toml</code>.
	</p>

	<label class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1" for="pkg-version">Version override (optional)</label>
	<input
		id="pkg-version"
		bind:value={version}
		placeholder="e.g. 0.1.0 (defaults to typst.toml)"
		class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm mb-2 focus:border-[var(--color-accent)] focus:outline-none"
	/>

	{#if limitsLabel}
		<p class="text-xs text-[var(--color-ink-muted)] mt-1">{limitsLabel}</p>
	{/if}

	{#if blockedReason}
		<div class="text-sm text-[var(--color-danger)] mt-2">{blockedReason}</div>
	{/if}
	{#if error}
		<div class="text-sm text-[var(--color-danger)] mt-2 break-words">{error}</div>
	{/if}
	{#if success}
		<div class="text-sm text-[var(--color-success)] mt-2">{success}</div>
	{/if}

	{#snippet footer()}
		<button onclick={onClose} class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Close</button>
		<button onclick={publish} disabled={publishing || Boolean(blockedReason)} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50 flex items-center gap-2">
			{#if publishing}<Icon icon="mdi:loading" class="animate-spin" />{/if}
			Publish
		</button>
	{/snippet}
</Modal>
