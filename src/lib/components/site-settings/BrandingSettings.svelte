<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { brandingStore, defaultBranding, toSettings, type Branding, type BrandingSettings } from '#lib/ts/branding';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import LogoUploader from './branding/LogoUploader.svelte';

    const FALLBACK_COLOR_PICKER_VALUE = '#1e66f5';

    let form = $state<BrandingSettings | null>(null);
    let saved = $state<BrandingSettings | null>(null);
    let loading = $state(true);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(form !== null && saved !== null && JSON.stringify(form) !== JSON.stringify(saved));
    let matchesDefaults = $derived(form !== null && JSON.stringify(form) === JSON.stringify(toSettings(defaultBranding)));

    function applyBranding(branding: Branding) {
        brandingStore.set(branding);
        form = toSettings(branding);
        saved = toSettings(branding);
    }

    async function load() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/branding');
            if (res.ok) applyBranding(await res.json());
            else loadError = (await res.text()) || 'Failed to load branding.';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function save() {
        if (!form) return;
        saving = true;
        message = null;
        try {
            const res = await fetch('/api/admin/branding', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(form)
            });
            if (res.ok) {
                applyBranding(await res.json());
                message = { kind: 'success', text: 'Branding saved.' };
            } else {
                message = { kind: 'error', text: (await res.text()) || 'Failed to save branding.' };
            }
        } catch {
            message = { kind: 'error', text: 'Network error.' };
        }
        saving = false;
    }

    function discard() {
        if (saved) form = { ...saved };
        message = null;
    }

    function useDefaults() {
        form = toSettings(defaultBranding);
        message = null;
    }

    onMount(load);
</script>

<SettingsHeader
    title="Branding"
    description="Change the name, logo and colors people see across this server."
    icon="mdi:palette-swatch-outline"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if loading || !form}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        {#if $brandingStore.customized}
            <p class="flex items-start gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-4 py-3 text-xs text-[var(--color-ink-muted)]">
                <Icon icon="mdi:information-outline" class="mt-0.5 flex-shrink-0 text-base" />
                Branding differs from the defaults, so a small "Powered by TypstDrive" credit is shown in the corner of every page.
            </p>
        {/if}

        <SettingsCard title="Logo" description="Shown in the top bar, on sign-in pages and as the browser tab icon.">
            <SettingsRow label="Logo image" description="PNG, JPEG, GIF or WebP. You can crop it to a square after choosing it.">
                <LogoUploader />
            </SettingsRow>
        </SettingsCard>

        <SettingsCard title="Identity" description="Text and colors used throughout the site.">
            <SettingsRow label="Site name" description="Replaces TypstDrive in the top bar and page titles." forId="branding-site-name">
                <input id="branding-site-name" type="text" maxlength="40" bind:value={form.site_name} class={inputClass} />
            </SettingsRow>
            <SettingsRow label="Tagline" description="Short description used for link previews." forId="branding-tagline">
                <input id="branding-tagline" type="text" maxlength="120" bind:value={form.tagline} class="{inputClass} sm:!max-w-sm" />
            </SettingsRow>
            <SettingsRow label="Accent color" description="Used for buttons and highlights. Leave on the theme color to follow each user's theme.">
                <div class="flex items-center gap-2">
                    <input
                        type="color"
                        aria-label="Accent color"
                        value={form.accent_color || FALLBACK_COLOR_PICKER_VALUE}
                        oninput={(e) => form && (form.accent_color = e.currentTarget.value)}
                        class="h-9 w-12 cursor-pointer rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-1"
                    />
                    {#if form.accent_color}
                        <span class="font-mono text-xs text-[var(--color-ink-muted)]">{form.accent_color}</span>
                        <button type="button" onclick={() => form && (form.accent_color = '')} class="text-xs font-semibold text-[var(--color-accent)] hover:underline">Use theme color</button>
                    {:else}
                        <span class="text-xs text-[var(--color-ink-muted)]">Theme color</span>
                    {/if}
                </div>
            </SettingsRow>
            <SettingsRow label="Sign-in message" description="Optional note shown on the sign-in page." forId="branding-login-message">
                <textarea id="branding-login-message" rows="3" maxlength="300" bind:value={form.login_message} placeholder="Welcome to our team's Typst server." class="{inputClass} sm:!max-w-sm resize-y"></textarea>
            </SettingsRow>
            {#snippet footer()}
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={useDefaults} disabled={matchesDefaults || saving} class={secondaryButtonClass}>Use defaults</button>
                <button onclick={discard} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
                <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                    <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                    {saving ? 'Saving...' : 'Save branding'}
                </button>
            {/snippet}
        </SettingsCard>
    </div>
{/if}
