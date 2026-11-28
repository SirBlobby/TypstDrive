const modalButtonBase =
    'inline-flex items-center justify-center gap-1.5 rounded-md px-4 py-2 text-sm font-medium transition disabled:cursor-not-allowed disabled:opacity-50';

export const modalCancelClass = `${modalButtonBase} border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]`;

export const modalPrimaryClass = `${modalButtonBase} bg-[var(--color-accent)] text-white shadow-sm hover:opacity-90`;

export const modalDangerClass = `${modalButtonBase} bg-[var(--color-danger)] text-white shadow-sm hover:opacity-90`;

export type ModalTone = 'accent' | 'danger';

export function modalConfirmClass(tone: ModalTone) {
    return tone === 'danger' ? modalDangerClass : modalPrimaryClass;
}
