import { createEffect, createSignal, on, onCleanup, type Accessor } from 'solid-js';

/** Replay an opacity entry on a retained element; never remount its contents. */
export function createEntryMotion(key: Accessor<unknown>): Accessor<boolean> {
  const [entering, setEntering] = createSignal(false);
  createEffect(on(key, () => {
    setEntering(false);
    if (typeof document === 'undefined' || typeof requestAnimationFrame !== 'function') return;
    if (document.documentElement.hasAttribute('data-reduced-motion') || window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    // Schedule before the next paint; a second frame used to reveal content and
    // then fade it out, creating a visible flash on fast navigation.
    const frame = requestAnimationFrame(() => setEntering(true));
    onCleanup(() => cancelAnimationFrame(frame));
  }));
  return entering;
}
