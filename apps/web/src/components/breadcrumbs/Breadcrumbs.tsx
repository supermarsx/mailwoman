// t95: the one breadcrumb trail of the app (admin console and user Settings).
//
// `<nav aria-label>` around an ordered list; the last item is the current page
// (`aria-current="page"`, not a link). Earlier items are links when they have an
// href, plain text otherwise. A label is never a raw key or id: callers resolve
// each level first (`screens/Admin/shell/crumbLabels.ts`,
// `screens/Settings/settingsCrumbs.ts`); a level whose name is still being
// fetched passes `label: null` and shows a placeholder with a "Loading" text for
// assistive technology. Long labels are cut with an ellipsis and the full text
// is offered as a tooltip (only when it is actually cut). Labels are `dir="auto"`
// so a name in another script keeps its own direction inside an RTL trail.

import { For, Show, type JSX } from 'solid-js';
import { Icon } from '../Icon.tsx';
import * as css from './breadcrumbs.css.ts';

export interface BreadcrumbItem {
  /** The level's name; `null` while it is being fetched. */
  label: string | null;
  /** Href with the deploy prefix; absent on the current page or a level with no page.
   *  A level with `onNavigate` and no href is a button. */
  href?: string;
  onNavigate?: () => void;
}

export interface BreadcrumbsProps {
  items: readonly BreadcrumbItem[];
  /** The landmark's name (e.g. "Breadcrumb"). */
  label: string;
  /** Read out for a level still loading. */
  loadingLabel: string;
  class?: string;
  testId?: string;
}

/** Offer the full text as a tooltip when, and only when, the ellipsis cut it. */
function syncTooltip(host: HTMLElement, text: HTMLElement, full: string): void {
  if (text.scrollWidth > text.clientWidth + 1) host.setAttribute('data-tooltip', full);
  else host.removeAttribute('data-tooltip');
}

export function Breadcrumbs(props: BreadcrumbsProps): JSX.Element {
  return (
    <Show when={props.items.length > 0}>
      <nav class={[css.nav, props.class ?? ''].join(' ').trim()} aria-label={props.label} data-testid={props.testId}>
        <ol class={css.list}>
          <For each={props.items}>
            {(item, i) => {
              const last = (): boolean => i() === props.items.length - 1;
              let text: HTMLSpanElement | undefined;
              const hover = (event: Event): void => {
                if (text && item.label) syncTooltip(event.currentTarget as HTMLElement, text, item.label);
              };
              // `plain`: the level is not a link or button, so the text itself is the
              // item (and, on the last level, carries `aria-current`).
              const name = (plain = false): JSX.Element => (
                <Show
                  when={item.label !== null}
                  fallback={
                    <span class={css.placeholder} data-crumb-loading="" aria-busy="true" aria-current={plain && last() ? 'page' : undefined}>
                      <span class={css.srOnly}>{props.loadingLabel}</span>
                    </span>
                  }
                >
                  <span
                    ref={text}
                    class={`${css.label} ${last() ? css.current : ''}`}
                    dir="auto"
                    data-crumb-label=""
                    aria-current={plain && last() ? 'page' : undefined}
                    onPointerOver={plain ? hover : undefined}
                  >
                    {item.label}
                  </span>
                </Show>
              );
              return (
                <li class={css.item}>
                  <Show when={i() > 0}>
                    <span class={css.separator} aria-hidden="true"><Icon name="chevron" /></span>
                  </Show>
                  <Show
                    when={!last() && item.href}
                    fallback={
                      <Show
                        when={!last() && item.onNavigate}
                        fallback={
                          name(true)
                        }
                      >
                        {/* A level with a page but no address (user Settings). */}
                        <button type="button" class={css.linkButton} aria-busy={item.label === null ? 'true' : undefined} onPointerOver={hover} onFocus={hover} onClick={() => item.onNavigate?.()}>
                          {name()}
                        </button>
                      </Show>
                    }
                  >
                    <a
                      href={item.href}
                      aria-busy={item.label === null ? 'true' : undefined}
                      onPointerOver={hover}
                      onFocus={hover}
                      onClick={(e) => {
                        if (!item.onNavigate || e.ctrlKey || e.metaKey || e.shiftKey || e.button !== 0) return;
                        e.preventDefault();
                        item.onNavigate();
                      }}
                    >
                      {name()}
                    </a>
                  </Show>
                </li>
              );
            }}
          </For>
        </ol>
      </nav>
    </Show>
  );
}
