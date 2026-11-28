<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/ui/settings';
    import { twoFactorAction } from '#lib/ts/account/two-factor';

    let { prompt, submitLabel, allowEmailResend = false, onSubmit, onCancel }: {
        prompt: string;
        submitLabel: string;
        allowEmailResend?: boolean;
        onSubmit: (code: string) => Promise<string | null>;
        onCancel: () => void;
    } = $props();

    let code = $state('');
    let busy = $state(false);
    let errorMsg = $state('');
    let infoMsg = $state('');

    async function submit(event: Event) {
        event.preventDefault();
        busy = true;
        errorMsg = (await onSubmit(code)) ?? '';
        busy = false;
    }

    async function sendEmailCode() {
        errorMsg = '';
        const result = await twoFactorAction('/email', {}, 'Could not send the code');
        if (result.ok) infoMsg = 'A code is on its way to your email.';
        else errorMsg = result.error;
    }
</script>

<form class="space-y-3" onsubmit={submit}>
    <label for="two-factor-entry" class="block text-sm text-[var(--color-ink-muted)]">{prompt}</label>
    <input id="two-factor-entry" bind:value={code} required autocomplete="one-time-code" placeholder="123456" class="{inputClass} font-mono" />
    {#if errorMsg}
        <p class="text-sm text-[var(--color-danger)]">{errorMsg}</p>
    {/if}
    {#if infoMsg}
        <p class="text-sm text-[var(--color-success)]">{infoMsg}</p>
    {/if}
    <div class="flex flex-wrap gap-2">
        <button type="submit" disabled={busy} class={primaryButtonClass}>
            {#if busy}<Icon icon="mdi:loading" class="animate-spin text-base" />{/if}
            {submitLabel}
        </button>
        {#if allowEmailResend}
            <button type="button" onclick={sendEmailCode} class={secondaryButtonClass}>
                <Icon icon="mdi:email-fast-outline" class="text-base" />
                Email me a code
            </button>
        {/if}
        <button type="button" onclick={onCancel} class={secondaryButtonClass}>Cancel</button>
    </div>
</form>
