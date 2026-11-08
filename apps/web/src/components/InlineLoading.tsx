// The one loading indicator for everything that is not the app boot, an account
// switch or the first visit of a top-level screen (see docs/ui-loading.md).
// A label plus the shared Spinner, drawn only after `delayMs` so fast work shows
// nothing; the text is in the DOM (polite status) from the start.
import { Show, type JSX } from 'solid-js';
import { t } from '../i18n/index.ts';
import { Spinner } from './spinner/Spinner.tsx';
import { createElapsed, PENDING_BOUND_MS, AsyncError } from './AsyncState.tsx';
import { BUSY_DELAY_MS, useDelayedBusy } from './useDelayedBusy.ts';
import * as css from './InlineLoading.css.ts';

export function InlineLoading(props: {
  message?: string;
  /** Defaults to true; pass a signal-reading expression to keep it mounted. */
  active?: boolean;
  delayMs?: number;
  /** When given, a wait past the shared bound turns into a failure with Retry. */
  onRetry?: (() => void) | undefined;
  class?: string;
}): JSX.Element {
  const visible = useDelayedBusy(() => props.active !== false, props.delayMs ?? BUSY_DELAY_MS);
  const timedOut = createElapsed(PENDING_BOUND_MS);
  return (
    <Show when={!(props.onRetry !== undefined && timedOut())} fallback={<AsyncError onRetry={props.onRetry} />}>
      <span class={`inline-loading ${css.root} ${props.class ?? ''}${visible() ? '' : ` ${css.quiet}`}`} role="status" aria-live="polite" data-testid="inline-loading" data-visible={visible() ? 'true' : 'false'}>
        <Show when={visible()}><Spinner /></Show>
        <span>{props.message ?? t('common-loading')}</span>
      </span>
    </Show>
  );
}
