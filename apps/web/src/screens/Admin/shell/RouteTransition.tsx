import { createEffect, createSignal, on, onCleanup, type JSX } from 'solid-js';
import { route } from '../../../styles/motion.css.ts';

/** Animate the retained page container, never remount drafts or move focus. */
export function RouteTransition(props: { routeKey: string; children: JSX.Element }): JSX.Element {
  const [entering, setEntering] = createSignal(false);
  createEffect(on(() => props.routeKey, () => {
    setEntering(false);
    if (document.documentElement.hasAttribute('data-reduced-motion') || window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    const frame = requestAnimationFrame(() => setEntering(true));
    onCleanup(() => cancelAnimationFrame(frame));
  }));
  return <div class={route} data-testid="admin-route-transition" data-mw-entering={entering()}>{props.children}</div>;
}
