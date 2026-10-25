<script lang="ts">
    import Icon from '@iconify/svelte';
    import { providerIcon, signInUrl, type SignInProvider } from '#lib/ts/auth-options';

    let { providers, redirect, showDivider = false }: {
        providers: SignInProvider[];
        redirect?: string;
        showDivider?: boolean;
    } = $props();
</script>

{#if providers.length > 0}
    {#if showDivider}
        <div class="flex items-center gap-3 text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">
            <span class="h-px flex-1 bg-[var(--color-line)]"></span>
            or
            <span class="h-px flex-1 bg-[var(--color-line)]"></span>
        </div>
    {/if}
    <div class="space-y-2">
        {#each providers as provider (provider.id)}
            <a
                href={signInUrl(provider.id, { redirect })}
                data-sveltekit-reload
                class="w-full flex justify-center items-center gap-2 py-2.5 px-4 text-sm font-semibold rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
            >
                <Icon icon={providerIcon(provider)} class="text-lg" />
                Continue with {provider.name}
            </a>
        {/each}
    </div>
{/if}
