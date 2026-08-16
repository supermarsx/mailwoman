// The three states every async surface owes the user: pending, failed, empty.
//
// Before this, "pending" was a bare `Loading…` string with no ceiling on it. A
// spinner that resolves in 200 ms and one that spins until the tab is closed are
// the same DOM, so nothing in the app — and nothing in a passing test — could
// tell them apart. Every pending state here is BOUNDED: past `timeoutMs` it stops
// claiming to be loading and becomes a failure the user can act on.
//
// All copy comes from `common.ftl`, which is statically imported into the entry
// chunk (`i18n/catalog.ts`), so these render correctly at boot before any lazy
// catalog has arrived — which matters, because the boot spinner is one of them.

import { createSignal, onCleanup, Show, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import { NetworkError } from '../api/client.ts';
import * as a11y from './mailA11y.css.ts';

/**
 * How long any single loading state may claim to be loading.
 *
 * Not a request timeout — the transport has none and this does not add one. It
 * bounds what the USER is shown: past this the surface stops asserting progress
 * it cannot verify and offers a way out. A request that completes afterwards
 * still resolves normally; retrying is the user's choice, not a cancellation.
 */
export const PENDING_BOUND_MS = 15_000;

/**
 * `true` once `ms` has elapsed since the caller mounted. The timer is cleared on
 * cleanup, so an unmounted surface cannot flip a signal nobody is reading.
 */
export function createElapsed(ms: number): () => boolean {
  const [elapsed, setElapsed] = createSignal(false);
  const timer = setTimeout(() => setElapsed(true), ms);
  onCleanup(() => clearTimeout(timer));
  return elapsed;
}

/** The human-facing message for a caught error: a transport failure reads as one. */
export function messageForError(error: unknown): string {
  return error instanceof NetworkError ? t('common-error-network') : t('common-error');
}

/**
 * A failed async surface: what went wrong, and a way to try again.
 *
 * `onRetry` is omitted rather than stubbed when a surface genuinely cannot retry
 * — an button that re-runs something guaranteed to fail the same way is worse
 * than no button, because it reads as a remedy.
 */
export function AsyncError(props: {
  error?: unknown;
  onRetry?: (() => void) | undefined;
  /** Overrides the derived message when a surface has something specific to say. */
  message?: string;
}): JSX.Element {
  return (
    <div class="async-state async-state--error" role="alert">
      <p class="async-state__message">{props.message ?? messageForError(props.error)}</p>
      <Show when={props.onRetry !== undefined}>
        <button
          type="button"
          class={`btn btn--ghost async-state__retry ${a11y.focusable}`}
          data-testid="async-retry"
          onClick={() => props.onRetry?.()}
        >
          {t('common-retry')}
        </button>
      </Show>
    </div>
  );
}

/** A surface that loaded and has nothing to show. Distinct from both others. */
export function AsyncEmpty(props: { message?: string }): JSX.Element {
  return (
    <p class="async-state async-state--empty" data-testid="async-empty">
      {props.message ?? t('common-empty')}
    </p>
  );
}

/**
 * A BOUNDED pending state. Renders as loading until `timeoutMs` elapses, then as
 * a failure with whatever way out the caller gave it.
 *
 * `role="status"` + `aria-live="polite"` so the transition is announced rather
 * than silently swapping under a screen reader.
 */
export function AsyncPending(props: {
  message?: string;
  timeoutMs?: number;
  onRetry?: (() => void) | undefined;
}): JSX.Element {
  const elapsed = createElapsed(props.timeoutMs ?? PENDING_BOUND_MS);
  return (
    <Show
      when={!elapsed()}
      fallback={<AsyncError onRetry={props.onRetry} />}
    >
      <p class="async-state async-state--pending" role="status" aria-live="polite" data-testid="async-pending">
        {props.message ?? t('common-loading')}
      </p>
    </Show>
  );
}
