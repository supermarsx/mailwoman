// Flicker guard for loading indicators: a busy flag that turns on only after
// `delayMs` of continuous work (fast work shows nothing at all) and, once on,
// stays on for at least `minMs` so it never blinks. Timers are cleared on cleanup.
import { createEffect, createSignal, onCleanup, type Accessor } from 'solid-js';

export const BUSY_DELAY_MS = 150;
export const BUSY_MIN_MS = 300;

export function useDelayedBusy(active: Accessor<boolean>, delayMs: number = BUSY_DELAY_MS, minMs: number = BUSY_MIN_MS): Accessor<boolean> {
  const [shown, setShown] = createSignal(false);
  let showTimer: ReturnType<typeof setTimeout> | undefined;
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let shownAt = 0;
  const clear = (): void => { clearTimeout(showTimer); clearTimeout(hideTimer); showTimer = undefined; hideTimer = undefined; };
  createEffect(() => {
    if (active()) {
      clearTimeout(hideTimer); hideTimer = undefined;
      if (shown() || showTimer !== undefined) return;
      if (delayMs <= 0) { shownAt = Date.now(); setShown(true); return; }
      showTimer = setTimeout(() => { showTimer = undefined; shownAt = Date.now(); setShown(true); }, delayMs);
    } else {
      clearTimeout(showTimer); showTimer = undefined;
      if (!shown()) return;
      const left = Math.max(0, minMs - (Date.now() - shownAt));
      if (left === 0) { setShown(false); return; }
      hideTimer = setTimeout(() => { hideTimer = undefined; setShown(false); }, left);
    }
  });
  onCleanup(clear);
  return shown;
}
