// "Restore previous value" for the last change of a setting, security rule,
// branding item or domain field (t32-ext.md §K).
//
// The previous value comes from the audit entry of that change (the server keeps
// `before`/`after` in `detail_json`, secrets redacted). Restoring is an ordinary
// audited write; this component only offers it and reports the result. Nothing is
// shown when there is no recorded previous value, and a redacted secret cannot be
// restored from here (the caller passes `restorable: false`).

import { Show, createSignal, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { Feedback, describeAdminError } from './Feedback.tsx';
import * as css from './shell.css.ts';

export interface PreviousValue {
  /** The previous value as shown to the admin. */
  display: string;
  /** Who made the change that replaced it. */
  actor: string;
  /** When, already formatted for the locale. */
  changedAt: string;
  /** False for a redacted secret: shown, but not offered for restore. */
  restorable?: boolean;
}

export interface RestorePreviousProps {
  previous: PreviousValue | null;
  onRestore: () => Promise<void>;
}

export function RestorePrevious(props: RestorePreviousProps): JSX.Element {
  const [busy, setBusy] = createSignal(false);
  const [ok, setOk] = createSignal<string | null>(null);
  const [err, setErr] = createSignal<string | null>(null);

  async function restore(): Promise<void> {
    setBusy(true);
    setOk(null);
    setErr(null);
    try {
      await props.onRestore();
      setOk(t('admin-shell-restore-done'));
    } catch (e) {
      setErr(describeAdminError(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Show when={props.previous}>
      {(prev) => (
        <div class={css.field} data-testid="restore-previous">
          <p class={css.help}>
            {t('admin-shell-restore-note', {
              actor: isolate(prev().actor),
              time: prev().changedAt,
              value: isolate(prev().display),
            })}
          </p>
          <Show when={prev().restorable !== false}>
            <div>
              <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void restore()}>
                {t('admin-shell-restore')}
              </button>
            </div>
          </Show>
          <Feedback kind="success" message={ok()} />
          <Feedback kind="error" as="toast" message={err()} />
        </div>
      )}
    </Show>
  );
}
