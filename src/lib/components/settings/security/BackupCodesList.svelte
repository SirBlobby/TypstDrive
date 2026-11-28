<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { brandingStore } from '#lib/ts/themes/branding';
    import { downloadBackupCodes } from '#lib/ts/account/two-factor';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/ui/settings';

    let { codes, onDone }: { codes: string[]; onDone: () => void } = $props();

    let copied = $state(false);

    async function copyCodes() {
        await navigator.clipboard.writeText(codes.join('\n'));
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }
</script>

<div class="space-y-4">
    <div class="flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-[var(--color-ink)]">
        <Icon icon="mdi:alert-outline" class="mt-0.5 flex-shrink-0 text-lg text-amber-500" />
        <p>Save these backup codes somewhere safe. Each one signs you in once if you lose your second factor. They are shown only now.</p>
    </div>
    <ul class="grid grid-cols-2 gap-2 rounded-md bg-[var(--color-surface-sunken)] p-4 font-mono text-sm text-[var(--color-ink)] sm:grid-cols-5">
        {#each codes as code (code)}
            <li>{code}</li>
        {/each}
    </ul>
    <div class="flex flex-wrap gap-2">
        <button type="button" onclick={copyCodes} class={secondaryButtonClass}>
            <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
            {copied ? 'Copied' : 'Copy'}
        </button>
        <button type="button" onclick={() => downloadBackupCodes(codes, $brandingStore.site_name)} class={secondaryButtonClass}>
            <Icon icon="mdi:download" class="text-base" />
            Download
        </button>
        <button type="button" onclick={onDone} class={primaryButtonClass}>I saved them</button>
    </div>
</div>
