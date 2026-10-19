<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';

    type AccountPolicy = {
        registration_enabled: boolean;
        allowed_email_domains: string[];
        allow_username_change: boolean;
        allow_email_change: boolean;
    };

    let policy = $state<AccountPolicy | null>(null);
    let saved = $state<AccountPolicy | null>(null);
    let domainsText = $state('');
    let loading = $state(true);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    function parseDomains(text: string) {
        return text
            .split(/[\s,]+/)
            .map((domain) => domain.trim().replace(/^@/, '').toLowerCase())
            .filter(Boolean);
    }

    let pendingPolicy = $derived(policy ? { ...policy, allowed_email_domains: parseDomains(domainsText) } : null);
    let hasChanges = $derived(pendingPolicy !== null && saved !== null && JSON.stringify(pendingPolicy) !== JSON.stringify(saved));

    function applyPolicy(value: AccountPolicy) {
        policy = { ...value };
        saved = { ...value, allowed_email_domains: [...value.allowed_email_domains] };
        domainsText = value.allowed_email_domains.join(', ');
    }

    async function load() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/admin/account-policy');
            if (res.ok) applyPolicy(await res.json());
            else loadError = (await res.text()) || 'Failed to load settings.';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function save() {
        if (!pendingPolicy) return;
        saving = true;
        message = null;
        try {
            const res = await fetch('/api/admin/account-policy', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(pendingPolicy)
            });
            if (res.ok) {
                applyPolicy(await res.json());
                message = { kind: 'success', text: 'Settings saved.' };
            } else {
                message = { kind: 'error', text: (await res.text()) || 'Failed to save settings.' };
            }
        } catch {
            message = { kind: 'error', text: 'Network error.' };
        }
        saving = false;
    }

    function discard() {
        if (saved) applyPolicy(saved);
        message = null;
    }

    onMount(load);
</script>

{#snippet toggle(label: string, checked: boolean, onToggle: () => void)}
    <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onclick={onToggle}
        class="relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors {checked ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface-sunken)]'}"
    >
        <span class="inline-block h-5 w-5 rounded-full bg-white shadow transition-transform {checked ? 'translate-x-5' : 'translate-x-0.5'}"></span>
    </button>
{/snippet}

<SettingsHeader
    title="Registration & Accounts"
    description="Decide who can sign up and what members can change about their own accounts. Administrators are not limited by the account rules."
    icon="mdi:account-cog-outline"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if loading || !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Registration" description="Controls the public sign-up page. Administrators can always create accounts from the Users section.">
            <SettingsRow label="Allow public registration" description="Let anyone with the link create an account.">
                {@render toggle('Allow public registration', policy.registration_enabled, () => policy && (policy.registration_enabled = !policy.registration_enabled))}
            </SettingsRow>
            <SettingsRow label="Allowed email domains" description="Only addresses at these domains can register or be set as an email. Separate with commas. Leave empty to allow any." forId="allowed-domains">
                <input id="allowed-domains" type="text" bind:value={domainsText} placeholder="example.com, school.edu" class="{inputClass} sm:!max-w-sm" />
            </SettingsRow>
        </SettingsCard>

        <SettingsCard title="Accounts" description="Rules that apply to members.">
            <SettingsRow label="Members can change their username" description="When off, only administrators can rename accounts.">
                {@render toggle('Members can change their username', policy.allow_username_change, () => policy && (policy.allow_username_change = !policy.allow_username_change))}
            </SettingsRow>
            <SettingsRow label="Members can change their email" description="When off, the email used at sign-up stays fixed.">
                {@render toggle('Members can change their email', policy.allow_email_change, () => policy && (policy.allow_email_change = !policy.allow_email_change))}
            </SettingsRow>
            {#snippet footer()}
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={discard} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
                <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                    <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                    {saving ? 'Saving...' : 'Save settings'}
                </button>
            {/snippet}
        </SettingsCard>
    </div>
{/if}
