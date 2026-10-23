// Breadcrumbs for nested admin screens (t32-ext.md §K).
// `<nav aria-label="Breadcrumb">` with an ordered list; the last item is the
// current page (`aria-current="page"`, not a link). Earlier items are links that
// navigate in place.

import { For, Show, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { Icon } from './icons.tsx';
import * as css from './shell.css.ts';

export interface Crumb {
  label: string;
  /** Href with the deploy prefix; absent on the current page. */
  href?: string;
  onNavigate?: () => void;
}

export interface BreadcrumbsProps {
  items: readonly Crumb[];
}

export function Breadcrumbs(props: BreadcrumbsProps): JSX.Element {
  return (
    <Show when={props.items.length > 0}>
      <nav class={css.crumbs} aria-label={t('admin-shell-breadcrumb')}>
        <ol class={css.crumbList}>
          <For each={props.items}>
            {(item, i) => {
              const last = (): boolean => i() === props.items.length - 1;
              return (
                <li class={css.crumbItem}>
                  <Show when={i() > 0}>
                    <Icon name="chevron" />
                  </Show>
                  <Show when={!last() && item.href} fallback={<span aria-current={last() ? 'page' : undefined}>{item.label}</span>}>
                    <a
                      href={item.href}
                      onClick={(e) => {
                        if (!item.onNavigate || e.ctrlKey || e.metaKey || e.shiftKey || e.button !== 0) return;
                        e.preventDefault();
                        item.onNavigate();
                      }}
                    >
                      {item.label}
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
