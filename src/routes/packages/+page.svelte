<script lang="ts">
	import { brandingStore } from '#lib/ts/branding';
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { userStore } from '#lib/ts/auth';
	import { fetchUserPackagePolicy, type Package, type UserPackagePolicy } from '#lib/ts/packages';
	import Navbar from '#lib/components/drive/Navbar.svelte';
	import DriveSidebar from '#lib/components/drive/DriveSidebar.svelte';
	import PackageCard from '#lib/components/packages/PackageCard.svelte';
	import ConfirmModal from '#lib/components/ConfirmModal.svelte';
	import PackageAccessModal from '#lib/components/packages/PackageAccessModal.svelte';
	import { activeTeamStore } from '#lib/ts/workspace';

	let packages = $state<Package[]>([]);
	let loading = $state(true);
	let searchQuery = $state('');
	let ownership = $state<'all' | 'mine'>('all');
	let packageToDelete = $state<Package | null>(null);
	let packageToShare = $state<Package | null>(null);
	let policy = $state<UserPackagePolicy | null>(null);

	let visiblePackages = $derived.by(() => {
		const query = searchQuery.trim().toLowerCase();
		return packages.filter((pkg) => {
			if (ownership === 'mine' && pkg.owner_id !== actingAccountId) return false;
			if (!query) return true;
			return pkg.name.toLowerCase().includes(query) || (pkg.description ?? '').toLowerCase().includes(query);
		});
	});

	let actingAccountId = $derived($activeTeamStore?.id ?? $userStore?.id);

	function canManage(pkg: Package) {
		return Boolean($userStore?.is_admin) || pkg.owner_id === actingAccountId;
	}

	function updateVisibility(pkg: Package, visibility: Package['visibility']) {
		packages = packages.map((item) => (item.id === pkg.id ? { ...item, visibility } : item));
	}

	function canDelete(pkg: Package) {
		if ($userStore?.is_admin) return true;
		return pkg.owner_id === actingAccountId && Boolean(policy?.can_delete_own);
	}

	async function load() {
		loading = true;
		try {
			const res = await fetch('/api/packages');
			packages = res.ok ? await res.json() : [];
		} catch {
			packages = [];
		}
		loading = false;
	}

	async function confirmRemove() {
		const pkg = packageToDelete;
		packageToDelete = null;
		if (!pkg) return;
		const res = await fetch(`/api/packages/${pkg.name}`, { method: 'DELETE' });
		if (res.ok) packages = packages.filter((p) => p.id !== pkg.id);
	}

	onMount(async () => {
		policy = await fetchUserPackagePolicy();
		await load();
	});
</script>

<svelte:head>
	<title>Packages - {$brandingStore.site_name}</title>
</svelte:head>

<div class="h-screen flex flex-col">
	<Navbar />

	<div class="flex flex-1 min-h-0">
		<DriveSidebar activeView="packages" />

		<main class="flex-1 min-w-0 overflow-y-auto px-6 lg:px-8 py-6">
			<header class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between mb-6">
				<div>
					<h1 class="text-2xl font-bold text-[var(--color-ink)]">Packages</h1>
					<p class="text-sm text-[var(--color-ink-muted)] mt-1">
						Typst packages you can use on this instance. Private packages are only shown to the people and teams they are shared with. Import them with
						<code class="font-mono text-xs bg-[var(--color-surface-sunken)] px-1.5 py-0.5 rounded">@typstdrive/name:version</code>.
					</p>
				</div>

				<div class="flex items-center gap-2">
					<div class="relative">
						<Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
						<input
							type="search"
							bind:value={searchQuery}
							placeholder="Search packages"
							aria-label="Search packages"
							class="w-64 bg-[var(--color-surface)] border border-[var(--color-line)] text-sm text-[var(--color-ink)] rounded-lg pl-9 pr-3 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
						/>
					</div>
					<div class="flex rounded-lg bg-[var(--color-surface-sunken)] p-0.5 text-xs font-medium">
						{#each [{ value: 'all', label: 'All' }, { value: 'mine', label: 'Mine' }] as option}
							<button
								onclick={() => (ownership = option.value as 'all' | 'mine')}
								class="px-3 py-1.5 rounded-md transition {ownership === option.value
									? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
									: 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
							>
								{option.label}
							</button>
						{/each}
					</div>
				</div>
			</header>

			{#if loading}
				<div class="min-h-[40vh] flex items-center justify-center text-[var(--color-ink-muted)]">
					<Icon icon="mdi:loading" class="text-3xl animate-spin" />
				</div>
			{:else if packages.length === 0}
				<div class="min-h-[50vh] flex flex-col items-center justify-center text-center">
					<div class="inline-flex items-center justify-center w-20 h-20 rounded-lg bg-purple-50 dark:bg-purple-500/10 text-purple-500 mb-5">
						<Icon icon="mdi:package-variant" class="text-4xl" />
					</div>
					<h3 class="text-lg font-semibold text-[var(--color-ink)] mb-1">No packages yet</h3>
					<p class="text-sm text-[var(--color-ink-muted)] max-w-sm">
						{policy && !policy.can_publish
							? 'Package publishing is not available for your account on this instance.'
							: 'Open a project and choose Publish to share it as a package everyone on this instance can import.'}
					</p>
				</div>
			{:else if visiblePackages.length === 0}
				<div class="min-h-[30vh] flex flex-col items-center justify-center text-center text-[var(--color-ink-muted)]">
					<Icon icon="mdi:package-variant-remove" class="text-4xl mb-3 opacity-60" />
					<p class="text-sm">No packages match your filters.</p>
				</div>
			{:else}
				<p class="text-xs text-[var(--color-ink-muted)] mb-3">{visiblePackages.length} {visiblePackages.length === 1 ? 'package' : 'packages'}</p>
				<div class="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-4 items-start">
					{#each visiblePackages as pkg (pkg.id)}
						<PackageCard
							{pkg}
							canDelete={canDelete(pkg)}
							canManage={canManage(pkg)}
							onDelete={(target) => (packageToDelete = target)}
							onManageAccess={(target) => (packageToShare = target)}
						/>
					{/each}
				</div>
			{/if}
		</main>
	</div>
</div>

{#if packageToShare}
	<PackageAccessModal
		pkg={packageToShare}
		canMakePrivate={Boolean(policy?.can_publish_private)}
		onClose={() => (packageToShare = null)}
		onChanged={(visibility) => packageToShare && updateVisibility(packageToShare, visibility)}
	/>
{/if}

{#if packageToDelete}
	<ConfirmModal
		title="Delete package"
		message={`'${packageToDelete.name}' and all of its versions will be permanently deleted. This cannot be undone.`}
		confirmLabel="Delete"
		onconfirm={confirmRemove}
		onclose={() => (packageToDelete = null)}
	/>
{/if}
