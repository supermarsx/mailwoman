// A route-driven ARIA tablist (WAI-ARIA tabs, automatic activation). Each tab is a
// link to its deep-link path (`#/users/<id>/<tab>`), so middle-click and "copy link"
// work; arrow keys, Home and End move focus and select (roving tabindex). The strip
// scrolls sideways when it does not fit and fades its clipped edges. Panels are the
// caller's: give each `id={tabPanelId(prefix, id)}` and `aria-labelledby={tabId(prefix, id)}`.

import { Index, Show, createEffect, createSignal, on, onCleanup, onMount, type JSX } from 'solid-js';
import { hrefFor } from './router.ts';
import { horizontalWheel } from '../../../components/horizontalWheel.ts';
import * as css from './routeTabs.css.ts';

export interface RouteTabItem { id: string; label: string; path: string; badge?: number | undefined }
export const tabId = (prefix: string, id: string): string => `${prefix}-tab-${id}`;
export const tabPanelId = (prefix: string, id: string): string => `${prefix}-panel-${id}`;

export function RouteTabs(props: { label: string; prefix: string; items: readonly RouteTabItem[]; selected: string; navigate: (path: string) => void }): JSX.Element {
  let scroller: HTMLElement | undefined;
  const [edges, setEdges] = createSignal({ start: false, end: false });
  const measure = (): void => {
    if (!scroller) return;
    const max = scroller.scrollWidth - scroller.clientWidth;
    const at = Math.abs(scroller.scrollLeft);
    setEdges({ start: at > 1, end: max - at > 1 });
  };
  onMount(() => {
    measure();
    const observer = typeof ResizeObserver === 'undefined' ? undefined : new ResizeObserver(measure);
    if (scroller) observer?.observe(scroller);
    onCleanup(() => observer?.disconnect());
  });
  // The selected tab stays in view when the strip scrolls (a deep link to a tab off the edge).
  createEffect(on(() => props.selected, (id) => {
    (document.getElementById(tabId(props.prefix, id)) as HTMLElement | null)?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
  }));
  function onKeyDown(event: KeyboardEvent): void {
    const keys = ['ArrowLeft', 'ArrowRight', 'Home', 'End'];
    if (!keys.includes(event.key) || event.ctrlKey || event.metaKey || event.altKey) return;
    const ids = props.items.map((item) => item.id);
    const from = ids.indexOf((event.target as HTMLElement).dataset['tab'] ?? props.selected);
    const rtl = getComputedStyle(event.currentTarget as HTMLElement).direction === 'rtl';
    const step = event.key === 'ArrowRight' ? (rtl ? -1 : 1) : event.key === 'ArrowLeft' ? (rtl ? 1 : -1) : 0;
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? ids.length - 1 : (from + step + ids.length) % ids.length;
    event.preventDefault();
    const item = props.items[next];
    if (!item) return;
    props.navigate(item.path);
    queueMicrotask(() => (document.getElementById(tabId(props.prefix, item.id)) as HTMLElement | null)?.focus());
  }
  return (
    <div class={css.frame} data-start={edges().start ? '' : undefined} data-end={edges().end ? '' : undefined}>
      <div class={css.scroller} ref={(el) => { scroller = el; horizontalWheel(el); }} onScroll={measure}>
        <div class={css.strip} role="tablist" aria-label={props.label} onKeyDown={onKeyDown}>
          <Index each={props.items}>{(entry) => { const item = (): RouteTabItem => entry(); return (
            <a
              class={css.tab}
              role="tab"
              id={tabId(props.prefix, item().id)}
              data-tab={item().id}
              href={hrefFor(item().path)}
              aria-selected={props.selected === item().id}
              aria-controls={tabPanelId(props.prefix, item().id)}
              tabindex={props.selected === item().id ? 0 : -1}
              onClick={(event) => {
                if (event.button !== 0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
                event.preventDefault();
                props.navigate(item().path);
              }}
            >
              {item().label}
              <Show when={(item().badge ?? 0) > 0}><span class={css.badge} data-testid={`tab-badge-${item().id}`}>{item().badge}</span></Show>
            </a>
          ); }}</Index>
        </div>
      </div>
    </div>
  );
}
