<script lang="ts">
    import { onMount } from 'svelte';
    import { userStore } from '#lib/ts/account/auth';
    import CodeEntry from './CodeEntry.svelte';
    import { twoFactorAction } from '#lib/ts/account/two-factor';

    let { onEnabled, onCancel }: { onEnabled: (backupCodes: string[]) => void; onCancel: () => void } = $props();

    let errorMsg = $state('');

    async function confirm(code: string) {
        const result = await twoFactorAction<{ backup_codes: string[] }>('/email/confirm', { code }, 'The code was not accepted');
        if (!result.ok) return result.error;
        onEnabled(result.value.backup_codes);
        return null;
    }

    onMount(async () => {
        const result = await twoFactorAction('/email', {}, 'Could not send the code');
        if (!result.ok) errorMsg = result.error;
    });
</script>

{#if errorMsg}
    <p class="mb-3 text-sm text-[var(--color-danger)]">{errorMsg}</p>
{/if}
<CodeEntry
    prompt="We emailed a 6-digit code to {$userStore?.email}. Enter it to turn on email codes."
    submitLabel="Turn on"
    allowEmailResend
    onSubmit={confirm}
    {onCancel}
/>
