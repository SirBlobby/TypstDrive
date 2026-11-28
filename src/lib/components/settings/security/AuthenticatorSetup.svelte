<script lang="ts">
    import { onMount } from 'svelte';
    import QrCode from '#lib/components/QrCode.svelte';
    import CodeEntry from './CodeEntry.svelte';
    import { twoFactorAction, type AuthenticatorSetup } from '#lib/ts/account/two-factor';

    let { onEnabled, onCancel }: { onEnabled: (backupCodes: string[]) => void; onCancel: () => void } = $props();

    let setup = $state<AuthenticatorSetup | null>(null);
    let errorMsg = $state('');

    async function confirm(code: string) {
        const result = await twoFactorAction<{ backup_codes: string[] }>('/authenticator/confirm', { code }, 'The code was not accepted');
        if (!result.ok) return result.error;
        onEnabled(result.value.backup_codes);
        return null;
    }

    onMount(async () => {
        const result = await twoFactorAction<AuthenticatorSetup>('/authenticator', {}, 'Could not start the setup');
        if (result.ok) setup = result.value;
        else errorMsg = result.error;
    });
</script>

{#if errorMsg}
    <p class="text-sm text-[var(--color-danger)]">{errorMsg}</p>
    <button type="button" onclick={onCancel} class="mt-2 text-xs font-semibold text-[var(--color-accent)] hover:underline">Back</button>
{:else if setup}
    <div class="space-y-5">
        <div class="flex flex-col gap-5 sm:flex-row sm:items-start">
            <QrCode size={setup.qr_size} modules={setup.qr_modules} label="QR code for your authenticator app" />
            <div class="min-w-0 space-y-2 text-sm text-[var(--color-ink-muted)]">
                <p>1. Scan this QR code with an authenticator app such as Google Authenticator, Microsoft Authenticator, 1Password or Bitwarden.</p>
                <p>2. If you cannot scan it, enter this key instead:</p>
                <code class="block break-all rounded bg-[var(--color-surface-sunken)] px-2 py-1.5 font-mono text-xs text-[var(--color-ink)]">{setup.secret}</code>
            </div>
        </div>
        <CodeEntry prompt="3. Enter the 6-digit code the app shows to finish." submitLabel="Turn on" onSubmit={confirm} {onCancel} />
    </div>
{:else}
    <p class="text-sm text-[var(--color-ink-muted)]">Preparing...</p>
{/if}
