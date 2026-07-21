<script lang="ts">
	import { page } from '$app/stores';
	import Icon from '@iconify/svelte';
</script>

<div class="min-h-[80vh] flex flex-col items-center justify-center p-4">
	<div class="text-center max-w-md bg-[var(--color-surface)] rounded-2xl shadow-xl border border-[var(--color-line)] p-8">
		<div class="inline-flex items-center justify-center w-20 h-20 rounded-full bg-[var(--color-danger)]/10 mb-6">
			<Icon icon="mdi:alert-circle-outline" class="h-10 w-10 text-[var(--color-danger)]" />
		</div>

		<h1 class="text-6xl font-bold text-[var(--color-ink)] mb-2 tracking-tight">
			{$page.status}
		</h1>

		<h2 class="text-xl font-semibold text-[var(--color-ink)] mb-4">
			Something went wrong
		</h2>

		<p class="text-base text-[var(--color-ink-muted)] mb-8 leading-relaxed">
			{$page.error?.message || 'We experienced an unexpected error processing your request.'}
		</p>

		<a
			href="/"
			class="inline-flex items-center justify-center gap-2 px-6 py-3 text-sm font-semibold rounded-xl shadow-sm text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition w-full"
		>
			<Icon icon="mdi:home" class="text-lg" />
			Return to Dashboard
		</a>
	</div>
</div>
