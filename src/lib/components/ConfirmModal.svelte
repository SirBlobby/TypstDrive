<script lang="ts">
  import Modal from "./Modal.svelte";

  interface Props {
    title: string;
    message: string;
    confirmLabel?: string;
    onconfirm: () => void;
    onclose: () => void;
  }

  let {
    title,
    message,
    confirmLabel = "Delete",
    onconfirm,
    onclose,
  }: Props = $props();
</script>

<Modal {title} icon="ph:warning-circle" {onclose}>
  <p class="text-sm text-[var(--color-ink-muted)]">{message}</p>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      class="rounded-md bg-[var(--color-danger)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
      onclick={onconfirm}
    >
      {confirmLabel}
    </button>
  {/snippet}
</Modal>
