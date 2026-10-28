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

import { createSignal, onCleanup, Show, For, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import { NetworkError } from '../api/client.ts';
import * as a11y from './mailA11y.css.ts';
import * as loading from './asyncState.css.ts';
import {Icon} from './Icon.tsx';

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
  detail?: string;
  onReload?: (() => void) | undefined;
}): JSX.Element {
  return (
    <div class={`async-state async-state--error ${loading.errorCard}`} role="alert">
      <p class={loading.errorHeading}><svg class={loading.errorMark} viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 6v8m0 3v1" fill="none" stroke="currentColor" stroke-width="2"/></svg>{props.message ?? messageForError(props.error)}</p>
      <Show when={props.onRetry !== undefined}><p class={loading.errorHelp}>{props.detail??t(props.error instanceof NetworkError?'common-error-network-help':'common-error-retry-help')}</p></Show>
      <Show when={props.onRetry !== undefined}>
        <button
          type="button"
          class={`async-state__retry ${loading.retry} ${a11y.focusable}`}
          data-testid="async-retry"
          onClick={() => props.onRetry?.()}
        >
          <Icon name="refresh"/>{t('common-retry')}
        </button>
      </Show>
      <Show when={props.onReload}><button type="button" class={`${loading.retry} ${a11y.focusable}`} onClick={()=>props.onReload?.()}><Icon name="refresh"/>{t('common-reload-app')}</button></Show>
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
  /** `messages`: message-list rows (avatar, sender + date, subject, preview). */
  variant?: 'lines' | 'list' | 'table' | 'page' | 'boot' | 'status' | 'messages';
  timeoutMs?: number;
  onRetry?: (() => void) | undefined;
  /** Announce the message to assistive technology only; the shapes carry it visually. */
  quiet?: boolean;
  /**
   * `false` for a wait that the caller bounds itself from a real progress
   * signal (first-sync indexing), which can legitimately outlast `timeoutMs`.
   */
  bounded?: boolean;
  /** `messages`: the height of one list row, so placeholders match the density. */
  rowHeight?: number;
  /** `messages`: how many placeholder rows to draw. */
  rows?: number;
}): JSX.Element {
  const timedOut = createElapsed(props.timeoutMs ?? PENDING_BOUND_MS);
  const elapsed = (): boolean => props.bounded !== false && timedOut();
  return (
    <Show
      when={!elapsed()}
      fallback={<AsyncError onRetry={props.onRetry} />}
    >
      <div class={`async-state async-state--pending ${loading.pending} ${props.variant==='status'?'':loading.skeletonPanel} ${props.variant==='boot'?loading.boot:''} ${props.variant==='messages'?loading.messagesPanel:''}`} role="status" aria-live="polite" data-testid="async-pending">
        <span class={props.quiet ? a11y.srOnly : loading.indicator}>{props.message ?? t('common-loading')}</span>
        <Show when={props.variant==='messages'}><div aria-hidden="true" class={loading.messageSkeletons} data-skeleton="messages">
          <For each={Array.from({length:props.rows??8},(_,index)=>index)}>{(index)=> <div class={loading.messageSkeleton} style={{'block-size':`${props.rowHeight??72}px`}}>
            <div class={loading.messageSkeletonHead}><span class={loading.messageSkeletonAvatar}/><span class={loading.skeletonLine} style={{'max-inline-size':`${[42,30,48,36][index%4]}%`}}/><span class={loading.messageSkeletonDate}/></div>
            <span class={loading.skeletonLine} style={{'max-inline-size':`${[72,58,80,64][index%4]}%`}}/>
            <span class={`${loading.skeletonLine} ${loading.messageSkeletonDim}`} style={{'max-inline-size':`${[90,84,76,88][index%4]}%`}}/>
          </div>}</For>
        </div></Show>
        <Show when={props.variant!=='status'&&props.variant!=='messages'}><div aria-hidden="true" class={loading.skeletonBody} data-skeleton={props.variant??'lines'}>
          <Show when={props.variant==='page'||props.variant==='boot'}><div class={loading.skeletonCards}><For each={[1,2,3]}>{()=> <span class={loading.skeletonCard}/>}</For></div></Show>
          <For each={Array.from({length:props.variant==='list'||props.variant==='table'?5:3})}>{()=> <div class={loading.skeletonRow}>
            <Show when={props.variant==='list'}><span class={loading.skeletonAvatar}/></Show>
            <span class={loading.skeletonLine}/>
            <Show when={props.variant==='table'}><span class={loading.skeletonLine}/><span class={loading.skeletonShort}/></Show>
          </div>}</For>
        </div></Show>
      </div>
    </Show>
  );
}
