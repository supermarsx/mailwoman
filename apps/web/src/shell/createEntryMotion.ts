import { createEffect, createSignal, on, onCleanup, type Accessor } from 'solid-js';

/** Replay an opacity entry on a retained element; never remount its contents. */
export function createEntryMotion(key: Accessor<unknown>): Accessor<boolean> {
  const [entering, setEntering] = createSignal(false);
  createEffect(on(key, () => {
    setEntering(false);
    if (typeof document === 'undefined' || typeof requestAnimationFrame !== 'function') return;
    if (document.documentElement.hasAttribute('data-reduced-motion') || window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    // Leave one painted frame without the animation so repeat navigation really
    // restarts it, even when the browser coalesces DOM writes before a frame.
    let nextFrame: number | undefined;
    const frame = requestAnimationFrame(() => {
      nextFrame = requestAnimationFrame(() => setEntering(true));
    });
    onCleanup(() => {
      cancelAnimationFrame(frame);
      if (nextFrame !== undefined) cancelAnimationFrame(nextFrame);
    });
  }));
  return entering;
}
