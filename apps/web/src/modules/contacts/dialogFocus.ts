// Self-contained modal focus management for the contacts dialogs (t8-e2 keeps
// focus primitives per-area — it does NOT import the e3-owned
// src/components/a11y/**, to avoid a cross-executor build-dependency race).
//
// `wireDialogFocus(el, onClose)` traps Tab within the dialog, closes it on
// Escape, moves focus to the first control on open, and restores focus to the
// element that had it when the dialog opened. Call it from an `onMount` with the
// dialog element; call the returned disposer from `onCleanup`.

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function wireDialogFocus(el: HTMLElement, onClose: () => void): () => void {
  const restoreEl = (document.activeElement as HTMLElement | null) ?? null;
  const first = el.querySelector<HTMLElement>(FOCUSABLE);
  (first ?? el).focus();

  const onKeyDown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
      return;
    }
    if (e.key !== 'Tab') return;
    const nodes = Array.from(el.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (nodes.length === 0) return;
    const firstNode = nodes[0]!;
    const lastNode = nodes[nodes.length - 1]!;
    if (e.shiftKey && document.activeElement === firstNode) {
      e.preventDefault();
      lastNode.focus();
    } else if (!e.shiftKey && document.activeElement === lastNode) {
      e.preventDefault();
      firstNode.focus();
    }
  };

  el.addEventListener('keydown', onKeyDown);
  return () => {
    el.removeEventListener('keydown', onKeyDown);
    restoreEl?.focus?.();
  };
}
