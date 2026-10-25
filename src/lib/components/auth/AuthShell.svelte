<script lang="ts">
    import type { Snippet } from 'svelte';
    import Icon from '@iconify/svelte';
    import Footer from '#lib/components/Footer.svelte';
    import { brandingStore, logoUrl } from '#lib/ts/branding';

    let { title, subtitle, icon, children, footer }: {
        title: string;
        subtitle?: string;
        icon?: string;
        children: Snippet;
        footer?: Snippet;
    } = $props();
</script>

<div class="min-h-screen flex flex-col bg-[var(--color-surface-muted)]">
    <div class="flex-grow flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-md w-full space-y-8 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-10 shadow-2xl">
            <div class="text-center">
                {#if icon}
                    <div class="inline-flex items-center justify-center w-16 h-16 rounded-lg bg-[var(--color-accent-soft)] text-[var(--color-accent)] mb-6 mx-auto shadow-sm">
                        <Icon {icon} class="text-3xl" />
                    </div>
                {:else}
                    <div class="inline-flex items-center justify-center w-24 h-24 rounded-lg bg-[var(--color-accent)] mb-6 mx-auto shadow-sm">
                        <img src={logoUrl($brandingStore)} alt={$brandingStore.site_name} class="h-14 w-14 object-contain" />
                    </div>
                {/if}
                <h2 class="text-3xl font-extrabold tracking-tight text-[var(--color-ink)]">{title}</h2>
                {#if subtitle}
                    <p class="mt-2 text-sm text-[var(--color-ink-muted)] font-medium">{subtitle}</p>
                {/if}
            </div>

            {@render children()}

            {#if footer}
                <div class="text-sm text-center mt-6 pt-4 border-t border-[var(--color-line)]">
                    {@render footer()}
                </div>
            {/if}
        </div>
    </div>
    <Footer />
</div>
