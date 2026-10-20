<script lang="ts">
    import Modal from '$lib/components/Modal.svelte';

    let { user, onClose }: {
        user: { id: string; username: string };
        onClose: () => void;
    } = $props();

    let newPassword = $state('');
    let confirmPassword = $state('');
    let errorMsg = $state('');
    let saving = $state(false);
    let saved = $state(false);

    async function save(event: Event) {
        event.preventDefault();
        errorMsg = '';
        if (newPassword !== confirmPassword) {
            errorMsg = 'Passwords do not match.';
            return;
        }
        saving = true;
        const res = await fetch(`/api/admin/users/${user.id}/password`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ password: newPassword })
        });
        saving = false;
        if (res.ok) saved = true;
        else errorMsg = (await res.text()) || 'Failed to set password.';
    }
</script>

<Modal title={`Set password for ${user.username}`} icon="mdi:key-variant" onclose={onClose}>
    {#if saved}
        <p class="text-sm text-[var(--color-ink)]">The password was changed and {user.username} was signed out of every session. Share the new password with them securely.</p>
    {:else}
        <form id="set-password-form" class="space-y-3" onsubmit={save}>
            <p class="text-xs text-[var(--color-ink-muted)]">Use this when a user forgot their password and email reset is not available. They will be signed out everywhere.</p>
            <input type="password" bind:value={newPassword} required autocomplete="new-password" placeholder="New password" class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]" />
            <input type="password" bind:value={confirmPassword} required autocomplete="new-password" placeholder="Confirm new password" class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]" />
            {#if errorMsg}
                <p class="text-sm text-[var(--color-danger)]">{errorMsg}</p>
            {/if}
        </form>
    {/if}

    {#snippet footer()}
        {#if saved}
            <button type="button" onclick={onClose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">Done</button>
        {:else}
            <button type="button" onclick={onClose} class="rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Cancel</button>
            <button type="submit" form="set-password-form" disabled={saving} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50">
                {saving ? 'Saving...' : 'Set password'}
            </button>
        {/if}
    {/snippet}
</Modal>
