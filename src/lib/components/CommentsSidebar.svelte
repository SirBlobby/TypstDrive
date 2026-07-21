<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';
	import { userStore } from '../ts/auth';
	import { commentReference } from '../ts/store';

	let { docId, onClose } = $props<{ docId: string, onClose: () => void }>();

	type Comment = {
		id: string;
		document_id: string;
		user_id: string;
		content: string;
		resolved: boolean;
		created_at: string;
		author_name?: string;
	};

	let comments = $state<Comment[]>([]);
	let newCommentContent = $state('');
	let loading = $state(true);
	let error = $state('');

	async function fetchComments() {
		loading = true;
		try {
			const res = await fetch(`/api/docs/${docId}/comments`);
			if (!res.ok) throw new Error('Failed to load comments');
			comments = await res.json();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function postComment() {
		if (!newCommentContent.trim()) return;
		try {
			const res = await fetch(`/api/docs/${docId}/comments`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ content: newCommentContent })
			});
			if (!res.ok) throw new Error('Failed to post comment');
			const c: Comment = await res.json();
			comments = [...comments, c];
			newCommentContent = '';
		} catch (e: any) {
			alert(e.message);
		}
	}

	async function deleteComment(id: string) {
		if (!confirm('Are you sure you want to delete this comment?')) return;
		try {
			const res = await fetch(`/api/comments/${id}`, { method: 'DELETE' });
			if (!res.ok) throw new Error('Failed to delete comment');
			comments = comments.filter(c => c.id !== id);
		} catch (e: any) {
			alert(e.message);
		}
	}

	async function toggleResolve(comment: Comment) {
		try {
			const res = await fetch(`/api/comments/${comment.id}`, {
				method: 'PATCH',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ resolved: !comment.resolved })
			});
			if (!res.ok) throw new Error('Failed to update comment');
			const updated: Comment = await res.json();
			comments = comments.map(c => c.id === comment.id ? updated : c);
		} catch (e: any) {
			alert(e.message);
		}
	}

	function formatDate(dateStr: string) {
		return new Date(dateStr).toLocaleString(undefined, {
			month: 'short', day: 'numeric', hour: 'numeric', minute: '2-digit'
		});
	}

	$effect(() => {
		if ($commentReference) {
			newCommentContent = `> ${$commentReference}\n\n`;
			$commentReference = '';
		}
	});

	onMount(() => {
		fetchComments();
	});
</script>

<div class="fixed right-0 top-0 bottom-0 w-80 bg-[var(--theme-bg)] backdrop-blur-xl border-l shadow-2xl flex flex-col z-[70] transform transition-transform duration-300 border-[var(--theme-border)]">
	<div class="flex items-center justify-between px-4 py-3 border-b bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]">
		<div class="flex items-center gap-2">
			<Icon icon="mdi:comment-text-multiple-outline" class="text-lg" />
			<h2 class="text-sm font-semibold text-[var(--theme-text)]">Comments</h2>
			<span class="text-[10px] font-bold px-2 py-0.5 rounded-full">{comments.length}</span>
		</div>
		<button onclick={onClose} class="p-1.5 hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors" title="Close Comments">
			<Icon icon="mdi:close" class="text-lg" />
		</button>
	</div>

	<div class="flex-1 overflow-y-auto p-4 space-y-4">
		{#if loading}
			<div class="flex justify-center items-center h-full">
				<Icon icon="mdi:loading" class="animate-spin text-2xl" />
			</div>
		{:else if error}
			<div class="text-[var(--color-danger)] text-sm text-center p-4 bg-[var(--color-danger)]/10 rounded-md border border-[var(--color-danger)]/20">
				{error}
			</div>
		{:else if comments.length === 0}
			<div class="flex flex-col items-center justify-center h-full space-y-2">
				<Icon icon="mdi:comment-off-outline" class="text-4xl opacity-50" />
				<p class="text-sm">No comments yet</p>
			</div>
		{:else}
			{#each comments as comment}
				<div class="group flex flex-col gap-2 p-3 border rounded-xl shadow-sm hover:shadow-md transition-all {comment.resolved ? 'opacity-60' : ''} bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]">
					<div class="flex justify-between items-start">
						<div class="flex items-center gap-2">
							<div class="w-6 h-6 rounded-full bg-[var(--color-accent-soft)] text-[var(--color-accent)] flex items-center justify-center text-xs font-bold">
								{(comment.author_name || 'A').substring(0, 1).toUpperCase()}
							</div>
							<div>
								<p class="text-xs font-semibold text-[var(--theme-text)]">{comment.author_name || 'Anonymous'}</p>
								<p class="text-[10px]">{formatDate(comment.created_at)}</p>
							</div>
						</div>
						
						<div class="flex opacity-0 group-hover:opacity-100 transition-opacity gap-1">
							{#if $userStore?.id === comment.user_id}
								<button onclick={() => deleteComment(comment.id)} class="p-1 hover:text-[var(--color-danger)] rounded hover:bg-[var(--color-danger)]/10 transition-colors" title="Delete">
									<Icon icon="mdi:trash-can-outline" class="text-xs" />
								</button>
							{/if}
							<button onclick={() => toggleResolve(comment)} class="p-1 hover:text-[var(--color-success)] rounded hover:bg-[var(--color-success)]/10 transition-colors" title={comment.resolved ? "Reopen" : "Resolve"}>
								<Icon icon={comment.resolved ? "mdi:check-circle" : "mdi:check-circle-outline"} class="text-xs" />
							</button>
						</div>
					</div>
					<p class="text-sm leading-relaxed whitespace-pre-wrap">{comment.content}</p>
				</div>
			{/each}
		{/if}
	</div>

	<div class="p-4 border-t bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]">
		<div class="relative">
			<textarea
				bind:value={newCommentContent}
				placeholder="Add a comment..."
				class="w-full border text-[var(--theme-text)] text-sm rounded-md px-3 py-2.5 pr-10 focus:outline-none focus:border-[var(--color-accent)] resize-none min-h-[80px] bg-[var(--theme-bg)] border-[var(--theme-border)]"
				onkeydown={(e) => {
					if (e.key === 'Enter' && !e.shiftKey) {
						e.preventDefault();
						postComment();
					}
				}}
			></textarea>
			<button 
				onclick={postComment}
				disabled={!newCommentContent.trim()}
				class="absolute bottom-2.5 right-2.5 p-1.5 bg-[var(--color-accent)] hover:opacity-90 disabled:bg-[var(--color-surface-sunken)] disabled:text-[var(--color-ink-muted)] rounded-md transition-colors"
				title="Post (Enter)"
			>
				<Icon icon="mdi:send" class="text-sm" />
			</button>
		</div>
		<p class="text-[10px] mt-2 text-center">Press <kbd class="font-mono px-1 py-0.5 rounded">Enter</kbd> to post, <kbd class="font-mono px-1 py-0.5 rounded">Shift+Enter</kbd> for newline</p>
	</div>
</div>