// Operation feedback and error wording (t32-admin.md §2.1, t32-ext.md §K).
//
// `Feedback` announces a result: success as `role=status` (polite), failure as
// `role=alert` (assertive). Each carries an icon and text, never colour alone.
//
// Where it shows (t32 f39, one toast system for webmail and console): a result
// of an action ("Saved", "Deleted", a failed request) is a toast
// (components/toast) and nothing stays in the page; a blocking state or a
// standing note (a load failure beside Try again, "not found", an error inside
// a dialog, "set by the environment") stays inline. `as` picks: success
// defaults to 'toast', error and info to 'inline', so a transient failure says
// `as="toast"`. The toast takes `testId` as its data-testid. With no
// ToastRegion mounted (a screen rendered on its own) the message stays inline,
// so a result is never raised where nobody can see it.
//
// `describeAdminError` turns a failed admin request into a sentence that says what
// failed and what to do. A server that sends `{code, detail}` gets its code mapped
// to a localized message with a next step; otherwise the HTTP status decides.

import { Show, createEffect, createUniqueId, on, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { toast, toastRegionMounted } from '../../../components/toast/index.ts';
import { Icon } from './icons.tsx';
import * as css from './shell.css.ts';

export type FeedbackKind = 'success' | 'error' | 'info';

export interface FeedbackProps {
  kind: FeedbackKind;
  /** Nothing is rendered while the message is empty or null. */
  message: string | null | undefined;
  testId?: string;
  /** 'toast': raised as a toast when the message is set, nothing rendered. Default: 'toast' for success, else 'inline'. */
  as?: 'inline' | 'toast';
}

export function Feedback(props: FeedbackProps): JSX.Element {
  const asToast = (): boolean =>
    (props.as ?? (props.kind === 'success' ? 'toast' : 'inline')) === 'toast' && toastRegionMounted();
  // One toast per Feedback: a second result from the same place replaces the first.
  const id = `feedback-${createUniqueId()}`;
  createEffect(
    on(
      () => [props.message, asToast()] as const,
      ([message, toToast]) => {
        if (!toToast || !message) return;
        toast.show(props.kind, message, { id, ...(props.testId ? { testId: props.testId } : {}) });
      },
    ),
  );
  return (
    <Show when={!asToast() && props.message}>
      {(message) => (
        <p
          class={[
            css.feedback,
            props.kind === 'success' ? css.feedbackOk : props.kind === 'error' ? css.feedbackErr : css.feedbackInfo,
          ].join(' ')}
          role={props.kind === 'error' ? 'alert' : 'status'}
          data-testid={props.testId}
        >
          <Icon name={props.kind === 'success' ? 'check' : props.kind === 'error' ? 'alert' : 'info'} />
          <span>{message()}</span>
        </p>
      )}
    </Show>
  );
}

/** Known server error codes and their messages (each names a next step). */
const CODE_MESSAGES: Record<string, () => string> = {
  upstream_dns_failed: () => t('admin-shell-err-upstream-dns'),
  upstream_connect_failed: () => t('admin-shell-err-upstream-connect'),
  upstream_tls_failed: () => t('admin-shell-err-upstream-tls'),
  upstream_auth_failed: () => t('admin-shell-err-upstream-auth'),
  capability_missing: () => t('admin-shell-err-forbidden'),
  domain_out_of_scope: () => t('admin-shell-err-out-of-scope'),
  rate_limited: () => t('admin-shell-err-rate-limited'),
  conflict: () => t('admin-shell-err-conflict'),
  validation_failed: () => t('admin-shell-err-validation'),
};

/** The parts of a failed request this module reads. `AdminApiError` has `status`;
 *  a client that parsed the body may add `code`. */
export interface ErrorLike {
  status?: number | undefined;
  code?: string | undefined;
  /** 429: seconds the server asked to wait (Retry-After). */
  retryAfterSecs?: number | undefined;
}

function asErrorLike(err: unknown): ErrorLike {
  if (typeof err !== 'object' || err === null) return {};
  const o = err as Record<string, unknown>;
  return {
    status: typeof o.status === 'number' ? o.status : undefined,
    code: typeof o.code === 'string' ? o.code : undefined,
    retryAfterSecs: typeof o.retryAfterSecs === 'number' && Number.isFinite(o.retryAfterSecs) ? o.retryAfterSecs : undefined,
  };
}

/** A localized "what failed, what to do" sentence for a failed admin request. */
export function describeAdminError(err: unknown): string {
  const { status, code, retryAfterSecs } = asErrorLike(err);
  // t104: a rate limit that says how long to wait says so, whichever limiter sent it.
  if (status === 429 && retryAfterSecs !== undefined) return t('admin-shell-err-rate-limited-in', { seconds: Math.max(1, Math.ceil(retryAfterSecs)) });
  const byCode = code !== undefined ? CODE_MESSAGES[code] : undefined;
  if (byCode) return byCode();
  if (status === undefined) return t('admin-shell-err-network');
  if (status === 401) return t('admin-shell-err-session');
  if (status === 403) return t('admin-shell-err-forbidden');
  if (status === 404) return t('admin-shell-err-not-found');
  if (status === 409 || status === 412) return t('admin-shell-err-conflict');
  if (status === 422 || status === 400) return t('admin-shell-err-validation');
  if (status === 429) return t('admin-shell-err-rate-limited');
  return t('admin-shell-err-server', { status });
}
