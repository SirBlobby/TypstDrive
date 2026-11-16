<script lang="ts">
  import Modal from "./Modal.svelte";
  import { splitConfirmMessage } from "#lib/ts/confirm-message";
  import { modalCancelClass, modalConfirmClass, type ModalTone } from "#lib/ts/modal-ui";

  interface Props {
    title: string;
    message: string;
    confirmLabel?: string;
    tone?: ModalTone;
    onconfirm: () => void;
    onclose: () => void;
  }

  let {
    title,
    message,
    confirmLabel = "Delete",
    tone = "danger",
    onconfirm,
    onclose,
  }: Props = $props();

  let parts = $derived(splitConfirmMessage(message));
  let icon = $derived(tone === "danger" ? "ph:trash" : "ph:question");
</script>

<Modal {title} {icon} {tone} {onclose}>
  <p class="text-sm leading-relaxed text-[var(--color-ink)]">
    {#each parts.lead as part, index (index)}
      {#if part.strong}<span class="font-semibold break-all">{part.text}</span>{:else}{part.text}{/if}
    {/each}
  </p>
  {#if parts.detail}
    <p class="mt-2 text-sm leading-relaxed text-[var(--color-ink-muted)]">{parts.detail}</p>
  {/if}

  {#snippet footer()}
    <!-- svelte-ignore a11y_autofocus -->
    <button autofocus class={modalCancelClass} onclick={onclose}>Cancel</button>
    <button class={modalConfirmClass(tone)} onclick={onconfirm}>{confirmLabel}</button>
  {/snippet}
</Modal>
