// A failed admin request, told in the corner toast (t104).
//
// The console's norm (Feedback.tsx, t32 f39): the result of an action is a
// toast in the ToastRegion, not a line in the page. A failed request is such a
// result, so it is raised here: one toast per `key` (a second failure from the
// same place replaces the first), the sentence `describeAdminError` gives
// ("Too many requests, try again in 12 s." for a 429), and a Retry action when
// the caller can run the request again. Retry respects Retry-After: pressed
// before the wait is over, it waits out the rest and then sends.
//
// Not for field validation (that stays at the field) or for a blocking state
// such as a dialog's own error; those keep `<Feedback>` inline. With no
// ToastRegion mounted (a screen rendered alone in a test) the helper returns
// false and the caller shows its inline fallback, so an error is never raised
// where nobody can see it.

import { t } from '../../../i18n';
import { TOAST_DURATIONS, toast, toastRegionMounted } from '../../../components/toast/index.ts';
import { describeAdminError } from './Feedback.tsx';

export interface RequestErrorToast {
  /** Stable per screen and action ("diag-backends"): one toast per key. */
  key: string;
  /** Runs the same request again; omitted, the toast has no Retry. */
  retry?: () => void | Promise<void>;
}

/** Seconds a 429 asked to wait, when it said. */
export function retryAfterOf(err: unknown): number | undefined {
  if (typeof err !== 'object' || err === null) return undefined;
  const o = err as { status?: unknown; retryAfterSecs?: unknown };
  return o.status === 429 && typeof o.retryAfterSecs === 'number' && Number.isFinite(o.retryAfterSecs)
    ? Math.max(0, Math.ceil(o.retryAfterSecs))
    : undefined;
}

/** The toast id for `key` (tests and callers that dismiss it on success). */
export const requestErrorId = (key: string): string => `admin-request-error:${key}`;

/**
 * Raise `err` as the corner error toast. False when no ToastRegion is mounted,
 * so the caller can keep its inline message.
 */
export function toastAdminError(err: unknown, options: RequestErrorToast): boolean {
  if (!toastRegionMounted()) return false;
  if ((err as { name?: unknown } | null)?.name === 'AbortError') return true;
  const wait = retryAfterOf(err);
  const due = wait === undefined ? 0 : Date.now() + wait * 1000;
  const retry = options.retry;
  toast.error(describeAdminError(err), {
    id: requestErrorId(options.key),
    testId: 'admin-request-error',
    // Long enough that Retry is still there when the wait is over.
    duration: Math.max(TOAST_DURATIONS.action, wait === undefined ? 0 : wait * 1000 + 5_000),
    ...(retry
      ? {
          action: {
            label: t('common-retry'),
            run: async () => {
              const left = due - Date.now();
              if (left > 0) await new Promise<void>((resolve) => { setTimeout(resolve, left); });
              await retry();
            },
          },
        }
      : {}),
  });
  return true;
}

/** The request worked: close its error toast, if one is up. */
export function clearAdminError(key: string): void {
  toast.dismiss(requestErrorId(key));
}

/**
 * The usual call from a screen: the corner toast, or `inline` (the screen's own
 * error line) when no ToastRegion is mounted. Field validation (422 with a
 * `field`) is not a toast: it goes to `inline`, beside the form.
 */
export function showAdminError(err: unknown, options: RequestErrorToast, inline: (message: string) => void): void {
  const field = (err as { field?: unknown } | null)?.field;
  if (typeof field === 'string' && field !== '') { inline(describeAdminError(err)); return; }
  if (!toastAdminError(err, options)) inline(describeAdminError(err));
}
