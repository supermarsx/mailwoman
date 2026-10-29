// One user-facing write, run with recovery (t82). The request layer
// (`api/recovery.ts`) already retries what is safe to retry; this is the UI end:
//   - no toast for an attempt that later succeeds;
//   - `onSlow` reports "Retrying…" / "Waiting for a connection…" only after
//     ~1 s (the request layer decides; the caller shows it next to the control);
//   - on the final failure, ONE error toast (a stable id per action, so a
//     second failure replaces it) with the human reason and a Retry action that
//     runs the same action again.
import { t } from '../i18n/index.ts';
import { toast } from './toast/toast.ts';
import { errorReason, type FailureReason } from '../api/recovery.ts';

export interface RecoverableAction {
  /** Stable per action and target: one toast per key. */
  key: string;
  run: () => Promise<void>;
  /** The toast text for a final failure, given the localised reason. */
  message: (reason: string) => string;
  /** Called on a final failure, after the toast (e.g. to reset a pending state). */
  onFailure?: (reason: FailureReason) => void;
}

/** The localised reason for a failure. */
export function failureText(reason: FailureReason): string {
  return t(`common-failure-${reason}`);
}

/** Run `action`; true when it succeeded. An abort (the user moved on) is silent. */
export async function runRecoverable(action: RecoverableAction): Promise<boolean> {
  const id = `recover:${action.key}`;
  try {
    await action.run();
    toast.dismiss(id);
    return true;
  } catch (error) {
    if ((error as { name?: unknown } | null)?.name === 'AbortError') return false;
    const reason = errorReason(error);
    toast.error(action.message(failureText(reason)), {
      id,
      action: { label: t('common-retry'), run: () => { void runRecoverable(action); } },
    });
    action.onFailure?.(reason);
    return false;
  }
}
