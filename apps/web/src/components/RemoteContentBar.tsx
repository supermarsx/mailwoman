// Remote-content (image-grant) bar (t16 §S8/S9, e14b) — the reader affordance for
// the anonymizing image proxy's deny-by-default remote images (SPEC §7.2). It is a
// STANDALONE, presentational component: it takes the blocked-content report + the
// message context + grant/revoke callbacks as props and owns no data-fetching, so
// the reader mounts it and wires the real `RemoteImageApi` (`api/remote-images.ts`)
// without this file knowing e6's wire shapes. The callbacks default to self-
// contained no-op mocks so the bar renders + tests on its own (mirrors
// `SecurityPanel`).
//
// Two states:
//   • blocked — the sanitizer stripped remote resources: show the count + the
//     hosts and offer the 4 grant scopes (this message / this sender / this domain
//     / all). Granting persists a 0016 grant, then the reader reloads the body so
//     the now-permitted images fetch through the proxy.
//   • allowed — a grant already covers the message: images load; offer "turn off"
//     (revoke) so the sender/domain/account goes back to blocking.
//
// Copy is factual (no-hype): it states exactly what was blocked and what each
// action loads. Untrusted host names carry `dir="auto"` so a spoofed RTL run can't
// reorder the row (SPEC §24).

import { For, Show, createMemo, createSignal, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import * as css from './remote-content-bar.css.ts';
import {
  scopeFor,
  senderDomain,
  type BlockedContentReport,
  type GrantScope,
  type GrantScopeKind,
  type RemoteImageGrant,
} from '../api/remote-images.ts';

export interface RemoteContentBarProps {
  /** The open message id (the `single`-scope grant value). */
  emailId: string;
  /** The sender address (drives the per-sender / per-domain scopes). */
  sender: string;
  /** What the sanitizer blocked in this body (from `analyzeBlockedContent`). */
  report: BlockedContentReport;
  /** An active grant already covering this message, or `null` when blocked. */
  activeGrant?: RemoteImageGrant | null;
  /** Persist a grant for a scope. Defaults to a no-op mock. */
  onGrant?: (scope: GrantScope) => Promise<void> | void;
  /** Revoke a grant. Defaults to a no-op mock. */
  onRevoke?: (scope: GrantScope) => Promise<void> | void;
}

/** The grant scopes offered, in order; `single` is the primary "load once". */
const GRANT_KINDS: GrantScopeKind[] = ['single', 'per-sender', 'per-domain', 'all'];

async function settle(v: Promise<void> | void): Promise<void> {
  if (v instanceof Promise) await v;
}

export function RemoteContentBar(props: RemoteContentBarProps): JSX.Element {
  const [pending, setPending] = createSignal(false);
  const [status, setStatus] = createSignal('');

  const ctx = createMemo(() => ({ emailId: props.emailId, sender: props.sender }));
  const domain = createMemo(() => senderDomain(props.sender));
  // per-sender needs an address; per-domain needs a domain. Hide the ones that
  // can't be expressed for this message rather than granting an empty scope.
  const kinds = createMemo<GrantScopeKind[]>(() =>
    GRANT_KINDS.filter((k) => {
      if (k === 'per-sender') return props.sender !== '';
      if (k === 'per-domain') return domain() !== '';
      return true;
    }),
  );

  function labelFor(kind: GrantScopeKind): string {
    if (kind === 'per-sender') return t('remote-grant-sender', { sender: props.sender });
    if (kind === 'per-domain') return t('remote-grant-domain', { domain: domain() });
    if (kind === 'all') return t('remote-grant-all');
    return t('remote-grant-once');
  }

  async function grant(kind: GrantScopeKind): Promise<void> {
    setPending(true);
    try {
      await settle((props.onGrant ?? (() => undefined))(scopeFor(kind, ctx())));
      setStatus(t(`remote-grant-done-${kind}`));
    } finally {
      setPending(false);
    }
  }

  async function revoke(): Promise<void> {
    const g = props.activeGrant;
    if (g === null || g === undefined) return;
    setPending(true);
    try {
      await settle(
        (props.onRevoke ?? (() => undefined))({ kind: g.scopeKind, value: g.scopeValue }),
      );
      setStatus(t('remote-revoke-done'));
    } finally {
      setPending(false);
    }
  }

  const summaryText = createMemo(() => {
    const r = props.report;
    // Prefer the tracker framing when the sanitizer classified any; otherwise the
    // neutral "remote images" count. Both are factual about what was stripped.
    return r.trackerCount > 0
      ? t('remote-blocked-trackers', { count: r.trackerCount, blocked: r.blockedCount })
      : t('remote-blocked-count', { count: r.blockedCount });
  });

  return (
    <section
      class={css.root}
      aria-label={t('remote-bar-label')}
      data-testid="remote-content-bar"
    >
      <Show
        when={props.activeGrant === null || props.activeGrant === undefined}
        fallback={
          <>
            <span class={css.summary}>{t('remote-allowed')}</span>
            <span class={css.actions}>
              <button
                type="button"
                class={`${css.btn} ${css.btnDanger}`}
                disabled={pending()}
                onClick={() => void revoke()}
              >
                {t('remote-revoke')}
              </button>
            </span>
          </>
        }
      >
        <span class={css.summary}>
          <span class={css.mark} aria-hidden="true">
            ⃠
          </span>
          {summaryText()}
        </span>
        <Show when={props.report.blockedHosts.length > 0}>
          <span class={css.hosts} aria-label={t('remote-blocked-hosts')}>
            <For each={props.report.blockedHosts}>
              {(h) => (
                <span class={css.host} dir="auto">
                  {h}
                </span>
              )}
            </For>
          </span>
        </Show>
        <span class={css.actions}>
          <For each={kinds()}>
            {(kind) => (
              <button
                type="button"
                class={`${css.btn} ${kind === 'single' ? css.btnPrimary : ''}`}
                disabled={pending()}
                onClick={() => void grant(kind)}
              >
                {labelFor(kind)}
              </button>
            )}
          </For>
        </span>
      </Show>
      <p class={css.status} role="status" aria-live="polite">
        {status()}
      </p>
    </section>
  );
}
