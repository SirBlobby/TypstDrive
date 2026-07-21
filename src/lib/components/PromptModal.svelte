<script lang="ts">
  import { untrack } from "svelte";
  import Modal from "./Modal.svelte";

  interface Props {
    title: string;
    label: string;
    icon?: string;
    value?: string;
    placeholder?: string;
    confirmLabel?: string;
    danger?: boolean;
    suffix?: string;
    onsubmit: (value: string) => void;
    onclose: () => void;
  }

  let {
    title,
    label,
    icon = "ph:pencil-simple",
    value = "",
    placeholder = "",
    confirmLabel = "Create",
    danger = false,
    suffix = "",
    onsubmit,
    onclose,
  }: Props = $props();

  let text = $state(
    untrack(() =>
      suffix && value.endsWith(suffix) ? value.slice(0, -suffix.length) : value,
    ),
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!text.trim()) return;
    onsubmit(text.trim());
  }
</script>

<Modal {title} {icon} {onclose}>
  <form id="prompt-form" onsubmit={submit}>
    <label class="flex flex-col gap-1 text-xs">
      <span class="font-medium text-[var(--color-ink-muted)]">{label}</span>
      <div
        class="flex items-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] focus-within:border-[var(--color-accent)]"
      >
        <!-- svelte-ignore a11y_autofocus -->
        <input
          autofocus
          class="min-w-0 flex-1 bg-transparent px-3 py-2 text-sm focus:outline-none"
          bind:value={text}
          {placeholder}
        />
        {#if suffix}
          <span class="pr-3 text-sm text-[var(--color-ink-muted)]">{suffix}</span>
        {/if}
      </div>
    </label>
  </form>

  {#snippet footer()}
    <button
      class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]"
      onclick={onclose}
    >
      Cancel
    </button>
    <button
      type="submit"
      form="prompt-form"
      class="rounded-md px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90
        {danger ? 'bg-[var(--color-danger)]' : 'bg-[var(--color-accent)]'}"
    >
      {confirmLabel}
    </button>
  {/snippet}
</Modal>
