<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';
	import Navbar from '$lib/components/dashboard/Navbar.svelte';
	import ProjectCard from '$lib/components/dashboard/ProjectCard.svelte';
	import PromptModal from '$lib/components/PromptModal.svelte';
	import ConfirmModal from '$lib/components/ConfirmModal.svelte';
	import Modal from '$lib/components/Modal.svelte';

	interface Project {
		id: string;
		name: string;
		entrypoint: string;
		thumbnail_svg?: string;
		updated_at: string;
		effective_role?: string;
	}

	let projects = $state<Project[]>([]);
	let shared = $state<Project[]>([]);
	let loading = $state(true);
	let showCreate = $state(false);
	let creating = $state(false);

	let activeMenu = $state<string | null>(null);
	let showRename = $state(false);
	let renameId = $state('');
	let renameName = $state('');
	let showInfo = $state(false);
	let infoProject = $state<Project | null>(null);
	let deleteTarget = $state<{ id: string; name: string } | null>(null);

	function setActiveMenu(id: string | null) { activeMenu = id; }
	function openInfo(project: Project) { activeMenu = null; infoProject = project; showInfo = true; }
	function openRename(id: string, name: string) { activeMenu = null; renameId = id; renameName = name; showRename = true; }

	async function submitRename(name: string) {
		const res = await fetch(`/api/projects/${renameId}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name })
		});
		if (res.ok) {
			projects = projects.map((p) => (p.id === renameId ? { ...p, name } : p));
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
			fetch('/api/projects').then((r) => (r.ok ? r.json() : [])),
			fetch('/api/projects/shared').then((r) => (r.ok ? r.json() : []))
		]);
		projects = own;
		shared = sh;
		loading = false;
	}

	async function create(name: string) {
		creating = true;
		const res = await fetch('/api/projects', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name })
		});
		creating = false;
		if (res.ok) {
			const project = await res.json();
			goto(`/project/${project.id}`);
		}
	}

	function requestDelete(id: string, name: string) {
		activeMenu = null;
		deleteTarget = { id, name };
	}

	async function confirmDeleteProject() {
		if (!deleteTarget) return;
		const res = await fetch(`/api/projects/${deleteTarget.id}`, { method: 'DELETE' });
		if (res.ok) projects = projects.filter((p) => p.id !== deleteTarget!.id);
		deleteTarget = null;
	}

	onMount(load);
</script>

<svelte:head>
	<title>Projects - TypstDrive</title>
</svelte:head>

<svelte:window onclick={handleWindowClick} />

<div class="min-h-screen bg-[var(--color-surface-muted)]">
	<Navbar />

	<div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
		<div class="flex items-center justify-between mb-6">
			<div>
				<button onclick={() => goto('/dashboard')} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors mb-2 flex items-center gap-1.5">
					<Icon icon="mdi:arrow-left" class="text-lg" />
					Back to Dashboard
				</button>
				<h2 class="text-2xl font-bold text-[var(--color-ink)] flex items-center gap-2">
					<Icon icon="mdi:folder-multiple-outline" class="text-[var(--color-accent)]" />
					Projects
				</h2>
				<p class="text-sm text-[var(--color-ink-muted)] mt-1">Multi-file Typst workspaces with their own <code class="font-mono text-xs">typst.toml</code>.</p>
			</div>
			<button onclick={() => (showCreate = true)} class="px-4 py-2 text-sm rounded-md bg-[var(--color-accent)] text-white hover:opacity-90 transition flex items-center gap-2">
				<Icon icon="mdi:plus" class="text-lg" /> New Project
			</button>
		</div>

		{#if loading}
			<p class="text-[var(--color-ink-muted)]">Loading…</p>
		{:else}
			{#if projects.length === 0}
				<div class="text-center py-16 text-[var(--color-ink-muted)]">
					<Icon icon="mdi:folder-multiple-outline" class="text-5xl mx-auto mb-3 opacity-50" />
					<p>No projects yet. Create one to start a multi-file project.</p>
				</div>
			{:else}
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each projects as project (project.id)}
						<ProjectCard
							{project}
							{activeMenu}
							{setActiveMenu}
							{openInfo}
							{openRename}
							deleteProject={requestDelete}
						/>
					{/each}
				</div>
			{/if}

			{#if shared.length > 0}
				<h3 class="text-lg font-semibold text-[var(--color-ink)] mt-10 mb-4 flex items-center gap-2">
					<Icon icon="mdi:account-group-outline" class="text-[var(--color-accent)]" /> Shared with me
				</h3>
				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
					{#each shared as project (project.id)}
						<button onclick={() => goto(`/project/${project.id}`)} class="text-left bg-[var(--color-surface)] rounded-lg border border-[var(--color-line)] overflow-hidden hover:border-[var(--color-accent)] transition">
							<div class="h-32 bg-[var(--color-surface-muted)] flex items-center justify-center overflow-hidden border-b border-[var(--color-line)]">
								{#if project.thumbnail_svg}
									{@html project.thumbnail_svg}
								{:else}
									<Icon icon="mdi:folder-multiple-outline" class="text-4xl text-[var(--color-ink-muted)]" />
								{/if}
							</div>
							<div class="p-3">
								<p class="font-medium text-[var(--color-ink)] truncate">{project.name}</p>
								<p class="text-xs text-[var(--color-ink-muted)] mt-0.5">{project.effective_role}</p>
							</div>
						</button>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

{#if showCreate}
	<PromptModal
		title="New project"
		label="Project name"
		icon="ph:folder-star"
		placeholder="Untitled Project"
		confirmLabel="Create"
		onsubmit={create}
		onclose={() => (showCreate = false)}
	/>
{/if}

{#if showRename}
	<PromptModal
		title="Rename project"
		label="Project name"
		icon="ph:pencil-simple"
		value={renameName}
		confirmLabel="Save"
		onsubmit={submitRename}
		onclose={() => (showRename = false)}
	/>
{/if}

{#if showInfo && infoProject}
	<Modal title={infoProject.name} icon="ph:folder-star" onclose={() => (showInfo = false)}>
		<div class="flex flex-col gap-4 text-xs">
			<div>
				<p class="mb-1 font-medium text-[var(--color-ink-muted)]">Entrypoint</p>
				<p class="font-mono text-sm text-[var(--color-ink)]">{infoProject.entrypoint}</p>
			</div>
			<div>
				<p class="mb-1 font-medium text-[var(--color-ink-muted)]">Last modified</p>
				<p class="text-sm text-[var(--color-ink)]">{new Date(infoProject.updated_at.endsWith('Z') ? infoProject.updated_at : infoProject.updated_at + 'Z').toLocaleString()}</p>
			</div>
		</div>

		{#snippet footer()}
			<button
				onclick={() => (showInfo = false)}
				class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
			>
				Close
			</button>
		{/snippet}
	</Modal>
{/if}

{#if deleteTarget}
	<ConfirmModal
		title="Delete project"
		message={`'${deleteTarget.name}' will be permanently deleted. This cannot be undone.`}
		confirmLabel="Delete"
		onconfirm={confirmDeleteProject}
		onclose={() => (deleteTarget = null)}
	/>
{/if}
