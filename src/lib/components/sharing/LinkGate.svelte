<script lang="ts">
    import type { Snippet } from 'svelte';
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import { canOpenItem, fetchLinkStatus, unlockLink, type LinkKind, type LinkStatus } from '$lib/ts/share-links';

    let { kind, itemId, children }: { kind: LinkKind; itemId: string; children: Snippet } = $props();

    let gateState = $state<'checking' | 'open' | 'locked' | 'expired' | 'denied'>('checking');
    let link = $state<LinkStatus | null>(null);
    let password = $state('');
    let error = $state('');
    let unlocking = $state(false);

    onMount(async () => {
        if (await canOpenItem(kind, itemId)) {
            gateState = 'open';
            return;
        }
        link = await fetchLinkStatus(kind, itemId);
        if (link?.expired) gateState = 'expired';
        else if (link?.requires_password && !link.unlocked) gateState = 'locked';
        else gateState = 'denied';
    });

    async function unlock(e: Event) {
        e.preventDefault();
        unlocking = true;
        error = (await unlockLink(kind, itemId, password)) ?? '';
        unlocking = false;
        if (!error) window.location.reload();
    }
</script>

{#if gateState === 'open'}
    {@render children()}
{:else}
    <div class="min-h-screen flex items-center justify-center bg-[var(--color-surface-muted)] px-4">
        <div class="w-full max-w-sm rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-8 text-center shadow-xl">
            {#if gateState === 'checking'}
                <p class="text-sm text-[var(--color-ink-muted)] animate-pulse">Opening...</p>
            {:else if gateState === 'locked'}
                <Icon icon="mdi:lock-outline" class="mx-auto mb-3 text-4xl text-[var(--color-accent)]" />
                <h1 class="text-lg font-semibold text-[var(--color-ink)]">{link?.name || 'Protected link'}</h1>
                <p class="mt-1 text-sm text-[var(--color-ink-muted)]">This link is protected. Enter the password to open it.</p>
                <form onsubmit={unlock} class="mt-5 space-y-3">
                    <input type="password" required bind:value={password} placeholder="Password" class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                    {#if error}
                        <p class="text-xs text-[var(--color-danger)]">{error}</p>
                    {/if}
                    <button type="submit" disabled={unlocking} class="w-full rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-60">
                        {unlocking ? 'Checking...' : 'Open'}
                    </button>
                </form>
            {:else if gateState === 'expired'}
                <Icon icon="mdi:link-off" class="mx-auto mb-3 text-4xl text-[var(--color-ink-muted)]" />
                <h1 class="text-lg font-semibold text-[var(--color-ink)]">This link has expired</h1>
                <p class="mt-1 text-sm text-[var(--color-ink-muted)]">Ask the owner for a new link.</p>
            {:else}
                <Icon icon="mdi:file-lock-outline" class="mx-auto mb-3 text-4xl text-[var(--color-ink-muted)]" />
                <h1 class="text-lg font-semibold text-[var(--color-ink)]">You do not have access</h1>
                <p class="mt-1 text-sm text-[var(--color-ink-muted)]">Ask the owner to share it with you.</p>
                {#if !$userStore}
                    <a href={`/login?redirect=${encodeURIComponent(window.location.pathname)}`} class="mt-5 inline-block rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-white hover:opacity-90">Sign in</a>
                {/if}
            {/if}
        </div>
    </div>
{/if}
