import { createEffect, createSignal, onCleanup, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';

// The shared reversible-action toast (plan §1.5): every 10-second undo — tag,
// pin, snooze, follow-up, archive/trash/move, sweep — and the undo-send Cancel
// window surface here via `app.pendingUndo()`. Distinct from the transient
// status `Toast`; this one carries an action button + a live countdown and
// auto-commits when the window elapses.

/** Seconds left before the pending action auto-commits. */
function remainingSeconds(expiresAt: number, now: number): number {
  return Math.max(0, Math.ceil((expiresAt - now) / 1000));
}

export function UndoToast(): JSX.Element {
  const app = useApp();
  const [now, setNow] = createSignal(Date.now());

  // Tick only while something is pending, so idle app does no timer work.
  createEffect(() => {
    if (app.pendingUndo() === null) return;
    setNow(Date.now());
    const h = setInterval(() => setNow(Date.now()), 250);
    onCleanup(() => clearInterval(h));
  });

  return (
    <Show when={app.pendingUndo()}>
      {(pending) => (
        <div class="undo-toast" role="status" aria-live="polite">
          <span class="undo-toast__label">{pending().label}</span>
          <span class="undo-toast__count" aria-hidden="true">
            {remainingSeconds(pending().expiresAt, now())}s
          </span>
          <button
            type="button"
            class="btn btn--ghost undo-toast__action"
            onClick={() => void app.undoNow()}
          >
            {pending().actionLabel}
          </button>
          <button
            type="button"
            class="undo-toast__dismiss"
            aria-label="Dismiss"
            onClick={() => app.dismissUndo()}
          >
            ✕
          </button>
        </div>
      )}
    </Show>
  );
}
