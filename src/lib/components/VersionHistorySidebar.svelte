<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { text } from '../ts/yjs-setup';

	let { docId, onClose } = $props<{ docId: string, onClose: () => void }>();

	type DocumentVersion = {
		id: string;
		document_id: string;
		user_id: string;
		content: string;
		created_at: string;
		author_name?: string;
	};

	let versions = $state<DocumentVersion[]>([]);
	let loading = $state(true);
	let error = $state('');
	let previewVersion = $state<DocumentVersion | null>(null);

	async function fetchVersions() {
		loading = true;
		try {
			const res = await fetch(`/api/docs/${docId}/versions`);
			if (!res.ok) throw new Error('Failed to load versions');
			versions = await res.json();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	function restoreVersion(version: DocumentVersion) {
		if (!text) return;
		if (!confirm('Are you sure you want to restore this version? This will overwrite the current document.')) return;
		
		const currentLength = text.length;
		text.delete(0, currentLength);
		text.insert(0, version.content);
		previewVersion = null;
		onClose();
	}

	function formatDate(dateStr: string) {
		return new Date(dateStr).toLocaleString(undefined, {
			month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit'
		});
	}

	onMount(() => {
		fetchVersions();
	});
</script>

<div class="fixed right-0 top-0 bottom-0 w-80 bg-[var(--theme-bg)] backdrop-blur-xl border-l shadow-2xl flex flex-col z-[70] transform transition-transform duration-300 border-[var(--theme-border)]">
	<!-- Header -->
	<div class="flex items-center justify-between px-4 py-3 border-b bg-gray-50/50 bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]">
		<div class="flex items-center gap-2">
			<Icon icon="mdi:history" class="text-lg" />
			<h2 class="text-sm font-semibold text-[var(--theme-text)]">Version History</h2>
			<span class="text-[10px] font-bold px-2 py-0.5 rounded-full">{versions.length}</span>
		</div>
		<button onclick={onClose} class="p-1.5 hover:text-gray-600 dark:hover:text-white hover:bg-gray-200 dark:hover:bg-white/10 rounded-md transition-colors" title="Close Version History">
			<Icon icon="mdi:close" class="text-lg" />
		</button>
	</div>

	<!-- Feed -->
	<div class="flex-1 overflow-y-auto p-4 space-y-4">
		{#if loading}
			<div class="flex justify-center items-center h-full">
				<Icon icon="mdi:loading" class="animate-spin text-2xl" />
			</div>
		{:else if error}
			<div class="text-red-500 text-sm text-center p-4 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-900/30">
				{error}
			</div>
		{:else if versions.length === 0}
			<div class="flex flex-col items-center justify-center h-full space-y-2">
				<Icon icon="mdi:history" class="text-4xl opacity-50" />
				<p class="text-sm">No versions saved yet</p>
			</div>
		{:else}
			{#each versions as version}
				<div class="group flex flex-col gap-2 p-3 border rounded-xl shadow-sm hover:shadow-md transition-all bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]">
					<div class="flex justify-between items-start">
						<div class="flex items-center gap-2">
							<div class="w-6 h-6 rounded-full bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 flex items-center justify-center text-xs font-bold">
								{(version.author_name || 'A').substring(0, 1).toUpperCase()}
							</div>
							<div>
								<p class="text-xs font-semibold text-[var(--theme-text)]">{version.author_name || 'Anonymous'}</p>
								<p class="text-[10px]">{formatDate(version.created_at)}</p>
							</div>
						</div>
					</div>
					
					<div class="flex gap-2 mt-2">
						<button onclick={() => previewVersion = version} class="flex-1 px-3 py-1.5 hover:bg-gray-200 dark:hover:bg-white/20 text-xs font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5">
							<Icon icon="mdi:eye" class="text-sm" />
							Preview
						</button>
						<button onclick={() => restoreVersion(version)} class="flex-1 px-3 py-1.5 bg-purple-50 text-purple-700 hover:bg-purple-100 dark:bg-purple-900/20 dark:text-purple-400 dark:hover:bg-purple-900/40 text-xs font-medium rounded-lg transition-colors flex items-center justify-center gap-1.5">
							<Icon icon="mdi:restore" class="text-sm" />
							Restore
						</button>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>

{#if previewVersion}
	<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4 transition-opacity" role="presentation" onclick={() => previewVersion = null} onkeydown={(e) => { if (e.key === "Escape") previewVersion = null; }}>
		<div class="bg-[var(--theme-bg)] backdrop-blur-xl rounded-2xl shadow-2xl border border-[var(--theme-border)] w-full max-w-4xl h-[80vh] flex flex-col transform transition-all" role="dialog" tabindex="-1" aria-modal="true" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
			<div class="flex items-center justify-between p-4 border-b border-[var(--theme-border)]">
				<div class="flex items-center gap-3">
					<Icon icon="mdi:eye" class="text-blue-500 text-xl" />
					<h3 class="text-lg font-semibold text-[var(--theme-text)]">Previewing Version</h3>
					<span class="text-sm">{formatDate(previewVersion.created_at)}</span>
				</div>
				<button onclick={() => previewVersion = null} class="p-1.5 hover:text-gray-600 dark:hover:text-white hover:bg-gray-200 dark:hover:bg-white/10 rounded-md transition-colors" title="Close Preview">
					<Icon icon="mdi:close" class="text-xl" />
				</button>
			</div>
			
			<div class="flex-1 overflow-auto p-6 bg-gray-50/50 bg-[var(--theme-bg)] text-[var(--theme-text)]">
				<pre class="text-sm font-mono whitespace-pre-wrap word-break-break-word">{previewVersion.content}</pre>
			</div>
			
			<div class="p-4 border-t flex justify-end gap-3 bg-white/50 rounded-b-2xl border-[var(--theme-border)]">
				<button onclick={() => previewVersion = null} class="px-4 py-2 text-sm font-medium hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
					Close
				</button>
				<button onclick={() => restoreVersion(previewVersion!)} class="bg-purple-600 hover:bg-purple-700 px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm flex items-center gap-2">
					<Icon icon="mdi:restore" class="text-lg" />
					Restore This Version
				</button>
			</div>
		</div>
	</div>
{/if}
