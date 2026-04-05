<script lang="ts">
	import { onMount } from 'svelte';
	import Editor from '$lib/components/Editor.svelte';
	import Preview from '$lib/components/Preview.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import { text, initYjs, cleanupYjs } from '$lib/ts/yjs-setup';
	import { compileTypst } from '$lib/ts/typst-api';
	import type { Diagnostic } from '$lib/ts/typst-api';
	import { page } from '$app/stores';

	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let timeoutId: number | undefined;
	let initialized = $state(false);
	let documentTitle = $state('Untitled Document');

	function triggerCompile() {
		if (!text) return;
		const content = text.toString();
		const docId = $page.params.id;
		compileTypst(content, docId)
			.then((res) => {
				if (res.svgs) {
					svgs = res.svgs;
					errors = [];
				} else if (res.errors) {
					errors = res.errors;
				}
			})
			.catch((e) => {
				console.error('Compilation fetch failed', e);
				errors = [{ message: 'Network or Server Error compiling document.', severity: 'error' }];
			});
	}

	onMount(() => {
		const docId = $page.params.id;
		if (!docId) return;

		
		fetch(`/api/docs/${docId}`)
			.then(res => res.json())
			.then(doc => {
				if (doc && doc.title) {
					documentTitle = doc.title;
				}
			})
			.catch(err => console.error("Failed to fetch document title:", err));

		initYjs(docId);
		initialized = true;

		text?.observe(() => {
			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = window.setTimeout(triggerCompile, 500);
		});

		triggerCompile();

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			cleanupYjs();
		};
	});
</script>

<svelte:head>
	<title>{documentTitle} - TypstDrive</title>
	<meta name="description" content={`Editing ${documentTitle} in TypstDrive.`} />
	<meta property="og:title" content={`${documentTitle} - TypstDrive`} />
</svelte:head>

<div class="flex flex-col h-screen relative">
	<Toolbar title={documentTitle} docId={$page.params.id} />

	<main class="flex-1 flex flex-col md:flex-row overflow-hidden relative">
		
		<div class="w-full md:w-1/2 flex flex-col relative min-h-[50%] md:min-h-0 bg-transparent z-10 shadow-[1px_0_10px_rgba(0,0,0,0.05)] dark:shadow-[1px_0_10px_rgba(0,0,0,0.2)] border-r border-gray-200 dark:border-white/10">
			{#if initialized}
				<Editor />
			{/if}
		</div>

		
		<div class="w-full md:w-1/2 relative bg-white/50 dark:bg-black/20 min-h-[50%] md:min-h-0 flex flex-col">
			<Preview {svgs} />
			<ErrorBanner {errors} />
		</div>
	</main>
</div>
