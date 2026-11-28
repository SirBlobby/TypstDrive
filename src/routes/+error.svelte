<script lang="ts">
	import { page } from '$app/state';
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { userStore } from '#lib/ts/account/auth';
	import { brandingStore, logoUrl } from '#lib/ts/themes/branding';
	import { errorCopy } from '#lib/ts/ui/error-pages';

	let copy = $derived(errorCopy(page.status));
	let detail = $derived(page.error?.message && page.error.message !== 'Not Found' ? page.error.message : '');
	let signedIn = $derived(!!$userStore);
	let needsSignIn = $derived(!signedIn && (page.status === 401 || page.status === 403));
	let signInHref = $derived(`/login?redirect=${encodeURIComponent(page.url.pathname + page.url.search)}`);
	let canGoBack = $state(false);

	$effect(() => {
		canGoBack = window.history.length > 1;
	});

	const primaryButton = 'inline-flex items-center justify-center gap-2 rounded-lg bg-[var(--color-accent)] px-5 py-2.5 text-sm font-semibold text-white transition hover:opacity-90 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-accent)]';
	const secondaryButton = 'inline-flex items-center justify-center gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-5 py-2.5 text-sm font-semibold text-[var(--color-ink)] transition hover:bg-[var(--color-surface-sunken)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-accent)]';
</script>

<svelte:head>
	<title>{copy.title} - {$brandingStore.site_name}</title>
</svelte:head>

<main class="flex min-h-screen flex-col items-center justify-center px-4 py-16">
	<a href="/" class="mb-10 flex items-center gap-2.5 text-[var(--color-ink)]">
		<img src={logoUrl($brandingStore)} alt="" class="h-8 w-8 rounded-lg" />
		<span class="text-lg font-semibold tracking-tight">{$brandingStore.site_name}</span>
	</a>

	<section class="w-full max-w-lg rounded-2xl border border-[var(--color-line)] bg-[var(--color-surface)] p-8 shadow-xl sm:p-10">
		<div class="mb-6 flex items-center gap-4">
			<div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-[var(--color-accent-soft)]">
				<Icon icon={copy.icon} class="text-2xl text-[var(--color-accent)]" />
			</div>
			<p class="font-mono text-sm font-semibold text-[var(--color-ink-muted)]">Error {page.status}</p>
		</div>

		<h1 class="mb-3 text-2xl font-bold tracking-tight text-[var(--color-ink)] sm:text-3xl">{copy.title}</h1>
		<p class="leading-relaxed text-[var(--color-ink-muted)]">{copy.description}</p>

		{#if detail}
			<p class="mt-4 rounded-lg bg-[var(--color-surface-muted)] px-3 py-2 text-sm text-[var(--color-ink)]">{detail}</p>
		{/if}

		{#if page.status === 404}
			<p class="mt-4 truncate rounded-lg bg-[var(--color-surface-muted)] px-3 py-2 font-mono text-xs text-[var(--color-ink-muted)]" title={page.url.pathname}>
				{page.url.pathname}
			</p>
		{/if}

		<div class="mt-8 flex flex-col gap-3 sm:flex-row">
			{#if needsSignIn}
				<a href={signInHref} class={primaryButton}>
					<Icon icon="mdi:login" class="text-lg" />
					Sign in
				</a>
			{:else if copy.canRetry}
				<button type="button" onclick={() => window.location.reload()} class={primaryButton}>
					<Icon icon="mdi:refresh" class="text-lg" />
					Try again
				</button>
			{:else}
				<a href={signedIn ? '/drive' : '/'} class={primaryButton}>
					<Icon icon="mdi:home-outline" class="text-lg" />
					{signedIn ? 'Go to your drive' : 'Go to the home page'}
				</a>
			{/if}

			{#if canGoBack}
				<button type="button" onclick={() => window.history.back()} class={secondaryButton}>
					<Icon icon="mdi:arrow-left" class="text-lg" />
					Go back
				</button>
			{/if}

			{#if copy.canRetry || needsSignIn}
				<a href={signedIn ? '/drive' : '/'} class="{secondaryButton} sm:ml-auto">
					<Icon icon="mdi:home-outline" class="text-lg" />
					Home
				</a>
			{/if}
		</div>
	</section>
</main>
