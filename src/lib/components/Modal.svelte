<script lang="ts">
  import Icon from "@iconify/svelte";
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    icon?: string;
    width?: string;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  }

  let {
    title,
    icon = "ph:squares-four",
    width = "max-w-md",
    onclose,
    children,
    footer,
  }: Props = $props();

  function handleKey(event: KeyboardEvent) {
    if (event.key === "Escape") onclose();
  }
</script>

<svelte:window on:keydown={handleKey} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-6"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <div
    class="w-full {width} overflow-hidden rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-label={title}
  >
    <header
      class="flex items-center gap-2 border-b border-[var(--color-line)] px-5 py-3.5"
    >
      <Icon {icon} class="text-lg text-[var(--color-accent)]" />
      <h2 class="flex-1 text-sm font-semibold">{title}</h2>
      <button
        class="rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
        onclick={onclose}
        aria-label="Close"
      >
        <Icon icon="ph:x" class="text-base" />
      </button>
    </header>

    <div class="scroll-thin max-h-[70vh] overflow-y-auto px-5 py-4">
      {@render children()}
    </div>

    {#if footer}
      <footer
        class="flex items-center justify-end gap-2 border-t border-[var(--color-line)] bg-[var(--color-surface-muted)] px-5 py-3"
      >
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>
