<script lang="ts">
    import { goto } from '$app/navigation';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }
</script>

<nav class="bg-[var(--color-surface)] shadow-sm border-b border-[var(--color-line)] px-4 py-2 flex justify-between items-center sticky top-0 z-10 transition-colors duration-200">
    <h1 class="text-lg font-bold text-[var(--color-ink)] flex items-center gap-2">
        <span class="flex h-9 w-9 items-center justify-center rounded-lg bg-[var(--color-accent)]">
            <img src="/favicon.png" alt="TypstDrive" class="h-6 w-6" />
        </span>
        TypstDrive
    </h1>
    <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5 text-[var(--color-ink-muted)] font-medium text-sm">
            <Icon icon="mdi:account-circle" class="text-lg" />
            {$userStore?.username}
        </div>
        <div class="h-5 w-px bg-[var(--color-line)]"></div>

        <a href="/projects" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Projects">
            <Icon icon="mdi:folder-multiple-outline" class="text-lg" />
        </a>
        <a href="/packages" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Packages">
            <Icon icon="mdi:package-variant-closed" class="text-lg" />
        </a>

        <a href="/api-docs" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="API Docs">
            <Icon icon="mdi:api" class="text-lg" />
        </a>
        <a href="https://typst.app/docs/" target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Typst Docs">
            <Icon icon="mdi:book-open-page-variant-outline" class="text-lg" />
        </a>
        <a href="https://typst.app/universe/" target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Typst Universe">
            <Icon icon="mdi:earth" class="text-lg" />
        </a>
        <div class="h-5 w-px bg-[var(--color-line)]"></div>

        <button onclick={() => goto('/settings')} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Settings">
            <Icon icon="mdi:cog" class="text-lg" />
        </button>
        <button onclick={logout} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-3 py-1.5 rounded-md flex items-center gap-2" title="Logout">
            <Icon icon="mdi:logout" class="text-base" />
        </button>
    </div>
</nav>
