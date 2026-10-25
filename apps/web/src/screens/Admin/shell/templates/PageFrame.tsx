// The frame every admin page template shares (t32-cfg UX3, UX11).
//
// `AdminPage`: the section heading (h2, the console's h1 is the brand), an
// optional one-line description and the page actions, then the content.
//
// `PageState`: the honest states, drawn inside the final layout so nothing jumps
// when data arrives (UX11):
//   • loading   — "Loading…" as a status;
//   • error     — what failed and what to do, the error code, and Try again;
//   • empty     — what is missing and the next step as a button;
//   • unavailable — "Not available — <reason>" for data the server does not have
//                   (a route that is not there yet, a feature switched off).

import { Match, Show, Switch, type JSX } from 'solid-js';
import { t } from '../../../../i18n';
import { describeAdminError } from '../Feedback.tsx';
import { Icon } from '../icons.tsx';
import * as ux from '../ux.css.ts';

export interface AdminPageProps {
  title: string;
  description?: string | undefined;
  /** Buttons at the top of the page ("Add a domain"). */
  actions?: JSX.Element;
  /** Id for the heading; templates point `aria-labelledby` at it. */
  titleId?: string | undefined;
  children: JSX.Element;
  testId?: string | undefined;
}

export function AdminPage(props: AdminPageProps): JSX.Element {
  const titleId = (): string => props.titleId ?? 'admin-page-title';
  return (
    <section class={ux.page} aria-labelledby={titleId()} data-testid={props.testId}>
      <header class={ux.pageHead}>
        <div class={ux.pageTitleBlock}>
          <h2 id={titleId()} class={ux.pageTitle}>
            {props.title}
          </h2>
          <Show when={props.description}>{(d) => <p class={ux.pageDescription}>{d()}</p>}</Show>
        </div>
        <Show when={props.actions}>
          <div class={ux.pageActions}>{props.actions}</div>
        </Show>
      </header>
      {props.children}
    </section>
  );
}

export type PageStateKind =
  | { kind: 'loading' }
  | { kind: 'error'; error: unknown; onRetry?: () => void }
  | { kind: 'empty'; text: string; action?: { label: string; onClick: () => void } }
  | { kind: 'unavailable'; reason: string };

/** The error code shown next to the sentence: the server's code, else the HTTP status. */
export function errorCode(err: unknown): string | null {
  if (typeof err !== 'object' || err === null) return null;
  const o = err as { code?: unknown; status?: unknown };
  if (typeof o.code === 'string' && o.code !== '') return o.code;
  if (typeof o.status === 'number') return `HTTP ${o.status}`;
  return null;
}

export function PageState(props: { state: PageStateKind; testId?: string }): JSX.Element {
  return (
    <div class={ux.pageState} data-testid={props.testId ?? 'page-state'} data-state={props.state.kind}>
      <Switch>
        <Match when={props.state.kind === 'loading'}>
          <p class={ux.pageStateText} role="status">
            {t('admin-shell-loading')}
          </p>
        </Match>
        <Match when={props.state.kind === 'error' ? props.state : undefined}>
          {(s) => (
            <>
              <p class={ux.pageStateText} role="alert">
                <Icon name="alert" /> {describeAdminError(s().error)}
              </p>
              <Show when={errorCode(s().error)}>
                {(code) => <p class={ux.pageStateCode}>{t('admin-ux-state-code', { code: code() })}</p>}
              </Show>
              <Show when={s().onRetry}>
                {(retry) => (
                  <button type="button" class="btn" onClick={() => retry()()}>
                    {t('admin-shell-retry')}
                  </button>
                )}
              </Show>
            </>
          )}
        </Match>
        <Match when={props.state.kind === 'empty' ? props.state : undefined}>
          {(s) => (
            <>
              <p class={ux.pageStateText}>{s().text}</p>
              <Show when={s().action}>
                {(a) => (
                  <button type="button" class="btn btn--primary" onClick={() => a().onClick()}>
                    {a().label}
                  </button>
                )}
              </Show>
            </>
          )}
        </Match>
        <Match when={props.state.kind === 'unavailable' ? props.state : undefined}>
          {(s) => (
            <p class={ux.pageStateText}>
              <Icon name="info" /> {t('admin-ux-state-unavailable', { reason: s().reason })}
            </p>
          )}
        </Match>
      </Switch>
    </div>
  );
}
