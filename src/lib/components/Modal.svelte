<script lang="ts">
  import Icon from "@iconify/svelte/dist/OfflineIcon.svelte";
  import type { Snippet } from "svelte";
  import type { ModalTone } from "#lib/ts/modal-ui";

  interface Props {
    title: string;
    icon?: string;
    tone?: ModalTone;
    description?: string;
    width?: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  let {
    title,
    icon = "ph:squares-four",
    tone = "accent",
    description,
    width = "max-w-md",
    onclose,
    children,
    footer,
  }: Props = $props();

  let badgeClass = $derived(
    tone === "danger"
      ? "bg-[color-mix(in_srgb,var(--color-danger)_14%,transparent)] text-[var(--color-danger)]"
      : "bg-[var(--color-accent-soft)] text-[var(--color-accent)]",
  );

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
  }
</script>

<svelte:window on:keydown={handleKey} />

<div
  class="fixed inset-0 z-[150] flex items-center justify-center bg-black/50 p-4 backdrop-blur-[2px] sm:p-6"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <div
    class="flex max-h-[90vh] w-full {width} flex-col overflow-hidden rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <header class="flex items-start gap-3 px-6 pt-5 pb-4">
      <span class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full {badgeClass}">
        <Icon {icon} class="text-lg" />
      </span>
      <div class="min-w-0 flex-1 pt-1.5">
        <h2 class="text-base leading-tight font-semibold">{title}</h2>
        {#if description}
          <p class="mt-1 text-sm text-[var(--color-ink-muted)]">{description}</p>
        {/if}
      </div>
      <button
        class="-mr-2 rounded-md p-1.5 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
        onclick={onclose}
        aria-label="Close"
      >
        <Icon icon="ph:x" class="text-base" />
      </button>
    </header>

    <div class="scroll-thin min-h-0 flex-1 overflow-y-auto px-6 pb-5">
      {@render children()}
    </div>

    {#if footer}
      <footer
        class="flex flex-wrap items-center justify-end gap-2 border-t border-[var(--color-line)] bg-[var(--color-surface-muted)] px-6 py-3.5"
      >
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>
