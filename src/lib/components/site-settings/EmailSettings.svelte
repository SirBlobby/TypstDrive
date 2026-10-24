<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass } from '$lib/ts/settings-ui';
    import { fetchAuthentication, sendTestEmail, type EmailStatus } from '$lib/ts/admin-auth';
    import { emailProviderGuides } from '$lib/ts/email-providers';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';

    let status = $state<EmailStatus | null>(null);
    let publicUrl = $state<string | null>(null);
    let recipient = $state('');
    let sending = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    async function sendTest(e: Event) {
        e.preventDefault();
        sending = true;
        message = null;
        try {
            const result = await sendTestEmail(recipient.trim());
            message = { kind: 'success', text: `Test email sent to ${result.sent_to}.` };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        sending = false;
    }

    onMount(async () => {
        const overview = await fetchAuthentication().catch(() => null);
        status = overview?.email ?? null;
        publicUrl = overview?.public_url ?? null;
    });
</script>

{#snippet value(text: string | number | null | undefined)}
    <span class="text-sm text-[var(--color-ink)] font-mono">{text ?? 'Not set'}</span>
{/snippet}

<SettingsHeader
    title="Email"
    description="Outgoing mail for verification, password resets and team invitations. Configure it with the SMTP_ environment variables."
    icon="mdi:email-outline"
/>

<div class="space-y-6">
    <SettingsCard title="Mail server">
        <SettingsRow label="Status">
            {#if status?.configured}
                <span class="inline-flex items-center gap-1.5 text-sm font-medium text-[var(--color-success)]"><Icon icon="mdi:check-circle-outline" /> Enabled</span>
            {:else}
                <span class="inline-flex items-center gap-1.5 text-sm font-medium text-[var(--color-ink-muted)]"><Icon icon="mdi:email-off-outline" /> Not configured</span>
            {/if}
        </SettingsRow>
        <SettingsRow label="Provider" description="SMTP_PROVIDER, or a custom server.">{@render value(status?.provider ?? (status?.host ? 'Custom server' : null))}</SettingsRow>
        <SettingsRow label="Server">{@render value(status?.host ? `${status.host}:${status.port}` : null)}</SettingsRow>
        <SettingsRow label="Encryption">{@render value(status?.security)}</SettingsRow>
        <SettingsRow label="Sender">{@render value(status?.from)}</SettingsRow>
        <SettingsRow label="Signs in to the server">{@render value(status ? (status.has_credentials ? 'Yes' : 'No') : null)}</SettingsRow>
        <SettingsRow label="Link address" description="PUBLIC_URL, used for links inside emails.">{@render value(publicUrl)}</SettingsRow>
        {#if status?.problem}
            <p class="px-5 py-3 text-xs text-[var(--color-danger)]">{status.problem}</p>
        {/if}
    </SettingsCard>

    <SettingsCard title="Send a test email" description="Leave the address empty to send it to your own email.">
        <form onsubmit={sendTest} class="flex flex-col gap-3 px-5 py-4 sm:flex-row sm:items-center">
            <input type="email" bind:value={recipient} placeholder="you@example.com" disabled={!status?.configured} class="{inputClass} sm:!max-w-none flex-1" />
            <button type="submit" disabled={!status?.configured || sending} class={primaryButtonClass}>
                <Icon icon={sending ? 'mdi:loading' : 'mdi:send-outline'} class="text-base {sending ? 'animate-spin' : ''}" />
                {sending ? 'Sending...' : 'Send test'}
            </button>
        </form>
        {#if message}
            <p class="px-5 pb-4 text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
        {/if}
    </SettingsCard>

    <SettingsCard
        title="Email providers"
        description="Set SMTP_PROVIDER to fill in the server for you, then add SMTP_USERNAME, SMTP_PASSWORD and SMTP_FROM. Providers with regions also read SMTP_REGION. For any other server, set SMTP_HOST instead."
    >
        <ul class="divide-y divide-[var(--color-line)]">
            {#each emailProviderGuides as guide (guide.value)}
                <li class="flex flex-col gap-1 px-5 py-3 sm:flex-row sm:items-start sm:gap-4">
                    <span class="w-36 shrink-0 text-sm font-medium text-[var(--color-ink)]">{guide.label}</span>
                    <span class="flex-1 text-xs text-[var(--color-ink-muted)]">
                        <code class="font-mono">SMTP_PROVIDER={guide.value}</code>
                        {#if guide.region}
                            <span class="ml-2"><code class="font-mono">SMTP_REGION</code> {guide.region}</span>
                        {/if}
                        <span class="block mt-1">{guide.credentials}</span>
                    </span>
                </li>
            {/each}
        </ul>
    </SettingsCard>
</div>
