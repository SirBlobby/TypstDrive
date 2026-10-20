<script lang="ts">
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass } from '$lib/ts/settings-ui';
    import { API_SCOPES, EXPIRY_CHOICES, type CreatedApiKey } from '$lib/ts/api-keys';

    let { maxRateLimit, onCreated }: { maxRateLimit: number; onCreated: (key: CreatedApiKey) => void } = $props();

    let name = $state('');
    let scopes = $state<string[]>(['render']);
    let expiresInDays = $state(0);
    let customRateLimit = $state<number | null>(null);
    let error = $state('');
    let creating = $state(false);

    function toggleScope(scope: string) {
        scopes = scopes.includes(scope) ? scopes.filter((item) => item !== scope) : [...scopes, scope];
    }

    async function submit(e: Event) {
        e.preventDefault();
        creating = true;
        error = '';
        try {
            const res = await fetch('/api/keys', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: name.trim(),
                    scopes,
                    expires_in_days: expiresInDays || null,
                    rate_limit: customRateLimit || null
                })
            });
            if (res.ok) {
                onCreated(await res.json());
                name = '';
            } else {
                error = (await res.text()) || 'Failed to create key.';
            }
        } catch {
            error = 'Network error.';
        }
        creating = false;
    }
</script>

<form onsubmit={submit} class="space-y-5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-5">
    <div>
        <label for="new-key-name" class="block text-sm font-medium text-[var(--color-ink)]">Key name</label>
        <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">A label to recognise where this key is used.</p>
        <input id="new-key-name" type="text" required bind:value={name} placeholder="e.g. CI pipeline" class="{inputClass} sm:!max-w-none mt-2" />
    </div>

    <fieldset>
        <legend class="text-sm font-medium text-[var(--color-ink)]">Scopes</legend>
        <div class="mt-2 grid gap-2 sm:grid-cols-2">
            {#each API_SCOPES as scope (scope.id)}
                <label class="flex cursor-pointer items-start gap-2 rounded-md border border-[var(--color-line)] p-3 text-sm {scopes.includes(scope.id) ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]' : ''}">
                    <input type="checkbox" checked={scopes.includes(scope.id)} onchange={() => toggleScope(scope.id)} class="mt-0.5" />
                    <span>
                        <span class="block font-medium text-[var(--color-ink)]">{scope.label}</span>
                        <span class="block text-xs text-[var(--color-ink-muted)]">{scope.description}</span>
                    </span>
                </label>
            {/each}
        </div>
    </fieldset>

    <div class="grid gap-4 sm:grid-cols-2">
        <div>
            <label for="new-key-expiry" class="block text-sm font-medium text-[var(--color-ink)]">Expires</label>
            <select id="new-key-expiry" bind:value={expiresInDays} class="{inputClass} mt-2">
                {#each EXPIRY_CHOICES as choice (choice.days)}
                    <option value={choice.days}>{choice.label}</option>
                {/each}
            </select>
        </div>
        <div>
            <label for="new-key-rate" class="block text-sm font-medium text-[var(--color-ink)]">Requests per minute</label>
            <input id="new-key-rate" type="number" min="1" max={maxRateLimit} bind:value={customRateLimit} placeholder={`Account limit (${maxRateLimit})`} class="{inputClass} mt-2" />
        </div>
    </div>

    <div class="flex items-center justify-end gap-3">
        {#if error}
            <p class="mr-auto text-xs text-[var(--color-danger)]">{error}</p>
        {/if}
        <button type="submit" disabled={creating || !name.trim() || scopes.length === 0} class={primaryButtonClass}>
            <Icon icon={creating ? 'mdi:loading' : 'mdi:key-plus'} class="text-base {creating ? 'animate-spin' : ''}" />
            {creating ? 'Creating...' : 'Create key'}
        </button>
    </div>
</form>
