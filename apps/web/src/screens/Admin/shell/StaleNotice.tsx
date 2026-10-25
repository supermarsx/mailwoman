// "Changed by someone else" notice for a refused write (t32-cfg UX19).
//
// Shown while `useEtag().conflict()` is set. It says who changed the object and
// when, and offers Reload (see their version, lose these edits) and Keep mine
// (save these values over theirs). An `alert`, so it is read when it appears;
// the buttons are plain buttons in the Tab order after the form's Save.

import { Show, createSignal, type JSX } from 'solid-js';
import { formatDateTime, isolate, t } from '../../../i18n';
import { Icon } from './icons.tsx';
import type { StaleConflict } from './useEtag.ts';
import * as css from './shell.css.ts';
import * as ux from './ux.css.ts';

export interface StaleNoticeProps {
  conflict: StaleConflict | null;
  onReload: () => void | Promise<void>;
  onKeepMine: () => void | Promise<void>;
}

export function StaleNotice(props: StaleNoticeProps): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  async function run(fn: () => void | Promise<void>): Promise<void> {
    setBusy(true);
    try {
      await fn();
    } catch {
      // The form reports the result of the repeated write itself.
    } finally {
      setBusy(false);
    }
  }
  const text = (c: StaleConflict): string => {
    const when = c.updatedAt ? formatDateTime(c.updatedAt) : '';
    if (c.updatedBy && when) return t('admin-ux-stale-by-at', { who: isolate(c.updatedBy), time: when });
    if (c.updatedBy) return t('admin-ux-stale-by', { who: isolate(c.updatedBy) });
    return t('admin-ux-stale-unknown');
  };
  return (
    <Show when={props.conflict}>
      {(c) => (
        <div class={[css.feedback, css.feedbackErr, ux.stale].join(' ')} role="alert" data-testid="stale-notice">
          <p class={ux.staleText}>
            <Icon name="alert" />
            <span>
              {text(c())} {t('admin-ux-stale-choice')}
            </span>
          </p>
          <div class={css.rowActions}>
            <button type="button" class="btn" disabled={busy()} onClick={() => void run(props.onReload)}>
              {t('admin-ux-stale-reload')}
            </button>
            <button type="button" class="btn btn--primary" disabled={busy()} onClick={() => void run(props.onKeepMine)}>
              {t('admin-ux-stale-keep')}
            </button>
          </div>
        </div>
      )}
    </Show>
  );
}
