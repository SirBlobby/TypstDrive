<script lang="ts">
    import Modal from '#lib/components/Modal.svelte';
    import { createTeam, type TeamSummary } from '#lib/ts/workspace';

    let { onClose, onCreated }: { onClose: () => void; onCreated: (team: TeamSummary) => void } = $props();

    let name = $state('');
    let error = $state('');
    let saving = $state(false);

    async function submit(e: Event) {
        e.preventDefault();
        saving = true;
        error = '';
        try {
            onCreated(await createTeam(name.trim()));
        } catch (err) {
            error = (err as Error).message;
        }
        saving = false;
    }
</script>

<Modal title="New team space" icon="mdi:account-group-outline" onclose={onClose}>
    <form id="create-team-form" onsubmit={submit} class="space-y-3">
        <p class="text-xs text-[var(--color-ink-muted)]">
            A team space has its own drive. You will be its owner and can invite people to work in it.
        </p>
        <label class="block text-sm font-medium text-[var(--color-ink)]">
            Team name
            <input
                type="text"
                required
                maxlength="80"
                bind:value={name}
                placeholder="e.g. Thesis group"
                class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
            />
        </label>
        {#if error}
            <p class="text-xs text-[var(--color-danger)]">{error}</p>
        {/if}
    </form>

    {#snippet footer()}
        <button onclick={onClose} class="rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Cancel</button>
        <button type="submit" form="create-team-form" disabled={saving || !name.trim()} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
            {saving ? 'Creating...' : 'Create team'}
        </button>
    {/snippet}
</Modal>
