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

/** t91: how a caller shows the failure toast (a slice passes its account-scoped `showToast`). */
export type FailureShow = (message: string, extra: { id: string; action: { label: string; run: () => void } }) => void;

const defaultShow: FailureShow = (message, extra) => { toast.error(message, extra); };

/**
 * t91: the one failure toast for an action the caller runs itself (a slice
 * action that returns a value and rethrows). `retry` repeats the SAME request
 * (the same body and the same idempotency key); a retry that fails again
 * replaces the toast, one that succeeds removes it. An abort shows nothing.
 */
export function showFailure(options: {
  key: string;
  error: unknown;
  message: (reason: string) => string;
  retry: () => Promise<unknown>;
  show?: FailureShow;
}): void {
  if ((options.error as { name?: unknown } | null)?.name === 'AbortError') return;
  const id = `recover:${options.key}`;
  const show = options.show ?? defaultShow;
  show(options.message(failureText(errorReason(options.error))), {
    id,
    action: {
      label: t('common-retry'),
      run: () => {
        options.retry().then(
          () => { toast.dismiss(id); },
          (error: unknown) => { showFailure({ ...options, error }); },
        );
      },
    },
  });
}

/** t91: remove an action's failure toast (it succeeded another way). */
export function dismissFailure(key: string): void {
  toast.dismiss(`recover:${key}`);
}

/** t91: the failure left it unknown whether the server got the request (worth a Retry). */
export function isTransientFailure(error: unknown): boolean {
  const reason = errorReason(error);
  return reason === 'network' || reason === 'server' || reason === 'offline' || reason === 'rate';
}
