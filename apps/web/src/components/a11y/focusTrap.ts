// Focus-trap + focus-restore + Esc-to-close primitive (plan §3 e3).
//
// This is the canonical, reusable dialog focus-management helper. A future
// consolidation pass points every web dialog/menu at THIS module; this batch the
// other web executors implement focus locally, so nothing imports it yet — it is
// built clean and exported for that migration.
//
// It is a SolidJS primitive: call it inside a component with an accessor for the
// container element (from a `ref`). While `active` is true it
//   • traps Tab / Shift+Tab inside the container (WCAG 2.2 — no keyboard trap out,
//     but a MODAL trap in, per the dialog pattern),
//   • moves focus to the container's initial target on activation,
//   • restores focus to the element that had it before activation on deactivation
//     (SC 2.4.3 Focus Order),
//   • invokes `onEscape` on the Esc key (dialog dismissal).
//
// jsdom-safe: all listeners are attached in an effect and cleaned up; importing
// this module has no side effects.

import { createEffect, onCleanup, type Accessor } from 'solid-js';
import { focusableWithin, firstFocusable, isFocusable } from './focusable.ts';

export interface FocusTrapOptions {
  /**
   * Whether the trap is armed. Defaults to always-on. Pass a signal so a dialog
   * can arm the trap only while open without re-mounting.
   */
  active?: Accessor<boolean>;
  /**
   * Element to focus when the trap activates. Defaults to the first focusable
   * descendant, falling back to the container itself (which should be
   * `tabindex="-1"` so it can receive focus).
   */
  initialFocus?: () => HTMLElement | null | undefined;
  /**
   * Restore focus to the previously-focused element on deactivation. Default true.
   */
  restoreFocus?: boolean;
  /** Called when Esc is pressed while the trap is active (e.g. close the dialog). */
  onEscape?: () => void;
}

const isBrowser = (): boolean => typeof document !== 'undefined';

/**
 * Arm a modal focus trap on `container` while `options.active` is true.
 *
 * ```tsx
 * let el!: HTMLDivElement;
 * const [open, setOpen] = createSignal(true);
 * createFocusTrap(() => el, { active: open, onEscape: () => setOpen(false) });
 * return <div ref={el} role="dialog" aria-modal="true" tabindex="-1">…</div>;
 * ```
 */
export function createFocusTrap(
  container: Accessor<HTMLElement | undefined | null>,
  options: FocusTrapOptions = {},
): void {
  if (!isBrowser()) return;

  createEffect(() => {
    const active = options.active ? options.active() : true;
    const el = container();
    if (!active || !el) return;

    const previouslyFocused = document.activeElement as HTMLElement | null;

    // Move focus inside on activation.
    const target = options.initialFocus?.() ?? firstFocusable(el) ?? el;
    // Focus on the next microtask so the element is laid out (dialogs mount then
    // trap); guard because the node may already be gone under fast toggles.
    queueMicrotask(() => {
      if (target.isConnected) target.focus();
    });

    const onKeyDown = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        if (options.onEscape) {
          e.preventDefault();
          options.onEscape();
        }
        return;
      }
      if (e.key !== 'Tab') return;

      const focusables = focusableWithin(el);
      if (focusables.length === 0) {
        // Nothing to move to — keep focus on the container.
        e.preventDefault();
        el.focus();
        return;
      }
      const first = focusables[0]!;
      const last = focusables[focusables.length - 1]!;
      const activeEl = document.activeElement as HTMLElement | null;

      // Wrap at the ends, and pull stray focus (e.g. on the container) back in.
      if (e.shiftKey) {
        if (activeEl === first || activeEl === el || !el.contains(activeEl)) {
          e.preventDefault();
          last.focus();
        }
      } else if (activeEl === last || activeEl === el || !el.contains(activeEl)) {
        e.preventDefault();
        first.focus();
      }
    };

    // Capture so we see the key before inner handlers can stop it.
    el.addEventListener('keydown', onKeyDown);

    onCleanup(() => {
      el.removeEventListener('keydown', onKeyDown);
      if (options.restoreFocus !== false && previouslyFocused && previouslyFocused.isConnected) {
        // Only restore if focus is still inside the trap (don't yank focus the
        // user has since moved elsewhere).
        const activeEl = document.activeElement as HTMLElement | null;
        if (activeEl === null || el.contains(activeEl) || activeEl === document.body) {
          if (isFocusable(previouslyFocused) || previouslyFocused.tabIndex >= 0) {
            previouslyFocused.focus();
          }
        }
      }
    });
  });
}
