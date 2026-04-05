<script lang="ts">
	import { page } from '$app/stores';
	import Icon from '@iconify/svelte';
</script>

<div class="min-h-[80vh] flex flex-col items-center justify-center p-4">
	<div class="text-center max-w-md bg-white dark:bg-zinc-900 rounded-2xl shadow-xl border border-gray-200 dark:border-zinc-800 p-8">
		<div class="inline-flex items-center justify-center w-20 h-20 rounded-full bg-red-100 dark:bg-red-500/10 mb-6">
			<Icon icon="mdi:alert-circle-outline" class="h-10 w-10 text-red-600 dark:text-red-500" />
		</div>
		
		<h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-2 tracking-tight">
			{$page.status}
		</h1>
		
		<h2 class="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-4">
			Something went wrong
		</h2>
		
		<p class="text-base text-gray-600 dark:text-gray-400 mb-8 leading-relaxed">
			{$page.error?.message || 'We experienced an unexpected error processing your request.'}
		</p>
		
		<a 
			href="/" 
			class="inline-flex items-center justify-center gap-2 px-6 py-3 border border-transparent text-sm font-semibold rounded-xl shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-zinc-900 transition-colors w-full"
		>
			<Icon icon="mdi:home" class="text-lg" />
			Return to Dashboard
		</a>
	</div>
</div>