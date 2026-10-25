<script lang="ts">
    import Icon from '@iconify/svelte';
    import { setStarred, starKey, starredStore, type StarKind } from '#lib/ts/stars';

    let { kind, id, name, sizeClass = 'text-base' }: {
        kind: StarKind;
        id: string;
        name: string;
        sizeClass?: string;
    } = $props();

    let starred = $derived($starredStore.has(starKey(kind, id)));
    let busy = $state(false);

    async function toggle(e: MouseEvent) {
        e.stopPropagation();
        if (busy) return;
        busy = true;
        await setStarred(kind, id, !starred);
        busy = false;
    }
</script>

<button
    type="button"
    onclick={toggle}
    onkeydown={(e) => e.stopPropagation()}
    disabled={busy}
    aria-pressed={starred}
    title={starred ? 'Remove from starred' : 'Add to starred'}
    aria-label={starred ? `Remove ${name} from starred` : `Add ${name} to starred`}
    class="p-1 rounded transition hover:bg-[var(--color-surface-sunken)] pointer-events-auto {starred
        ? 'text-amber-500'
        : 'text-[var(--color-ink-muted)] hover:text-amber-500 opacity-0 group-hover:opacity-100 focus-visible:opacity-100'}"
>
    <Icon icon={starred ? 'mdi:star' : 'mdi:star-outline'} class={sizeClass} />
</button>
