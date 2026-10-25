<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import { dangerButtonClass, formatDate, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { describeUserAgent, userAgentIcon } from '#lib/ts/user-agent';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';

    type Session = {
        id: string;
        user_agent: string | null;
        ip_address: string | null;
        created_at: string;
        last_seen_at: string;
        expires_at: string;
        current: boolean;
    };

    let sessions = $state<Session[]>([]);
    let loading = $state(true);
    let message = $state('');
    let busyId = $state<string | null>(null);

    let otherSessions = $derived(sessions.filter((session) => !session.current));

    async function loadSessions() {
        loading = true;
        const res = await fetch('/api/auth/sessions').catch(() => null);
        if (res?.ok) sessions = await res.json();
        else message = 'Failed to load sessions.';
        loading = false;
    }

    async function revoke(session: Session) {
        busyId = session.id;
        const res = await fetch(`/api/auth/sessions/${session.id}`, { method: 'DELETE' }).catch(() => null);
        busyId = null;
        if (!res?.ok) {
            message = 'Could not sign out that session.';
            return;
        }
        if (session.current) {
            userStore.set(null);
            goto('/login');
            return;
        }
        sessions = sessions.filter((item) => item.id !== session.id);
    }

    async function revokeOthers() {
        busyId = 'others';
        const res = await fetch('/api/auth/sessions/revoke-others', { method: 'POST' }).catch(() => null);
        busyId = null;
        if (res?.ok) {
            const result = await res.json();
            message = `Signed out ${result.revoked} other session(s).`;
            sessions = sessions.filter((session) => session.current);
        } else {
            message = 'Could not sign out the other sessions.';
        }
    }

    onMount(loadSessions);
</script>

<SettingsCard title="Active sessions" description="Browsers signed in to your account. Sign out any you do not recognise.">
    {#if loading}
        <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading sessions...</p>
    {:else}
        {#each sessions as session (session.id)}
            <div class="flex items-center gap-4 px-5 py-3">
                <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
                    <Icon icon={userAgentIcon(session.user_agent)} class="text-lg" />
                </div>
                <div class="min-w-0 flex-1">
                    <p class="flex items-center gap-2 truncate text-sm font-medium text-[var(--color-ink)]">
                        {describeUserAgent(session.user_agent)}
                        {#if session.current}
                            <span class="rounded bg-[var(--color-accent-soft)] px-1.5 py-0.5 text-[10px] font-semibold uppercase text-[var(--color-accent)]">This browser</span>
                        {/if}
                    </p>
                    <p class="text-xs text-[var(--color-ink-muted)]">
                        {session.ip_address ?? 'Unknown address'} · Signed in {formatDate(session.created_at)} · Last active {formatDate(session.last_seen_at)}
                    </p>
                </div>
                <button onclick={() => revoke(session)} disabled={busyId === session.id} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs flex-shrink-0">
                    <Icon icon="mdi:logout" class="text-sm" />
                    Sign out
                </button>
            </div>
        {/each}
    {/if}
    {#snippet footer()}
        {#if message}
            <p class="mr-auto text-xs text-[var(--color-ink-muted)]">{message}</p>
        {/if}
        <button onclick={revokeOthers} disabled={otherSessions.length === 0 || busyId === 'others'} class={dangerButtonClass}>
            <Icon icon="mdi:logout-variant" class="text-sm" />
            Sign out everywhere else
        </button>
    {/snippet}
</SettingsCard>
