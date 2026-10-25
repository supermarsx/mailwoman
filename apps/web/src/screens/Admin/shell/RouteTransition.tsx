import type { JSX } from 'solid-js';
import { createEntryMotion } from '../../../shell/createEntryMotion.ts';
import { route } from '../../../styles/motion.css.ts';

/** Animate the retained page container, never remount drafts or move focus. */
export function RouteTransition(props: { routeKey: string; children: JSX.Element }): JSX.Element {
  const entering = createEntryMotion(() => props.routeKey);
  return <div class={route} data-testid="admin-route-transition" data-mw-entering={entering()}>{props.children}</div>;
}
