// Admin sign-in gate (plan §2.5 — SEPARATE session domain). The panel is gated on
// an admin session distinct from the mailbox cookie; when none is present this
// form authenticates against `/admin/login` with the operator's username and
// password.
//
// An admin session ends after 30 minutes without a request and at most 12 hours
// after sign-in (`Store::ADMIN_SESSION_IDLE_SECS` / `ADMIN_SESSION_MAX_SECS`,
// `crates/mw-store/src/v6.rs`). The form states that, and says so when it is being
// shown because a session in use was ended.
//
// Second step (t32-ext.md §M): an admin account with TOTP gets a code field after
// the password is accepted; the session starts only when the code verifies
// (`shell/adminAuth.ts` has the wire contract). Without TOTP on the server this
// step never appears.

import { Show, createSignal, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import { AdminApiError } from '../../state/slices/admin.ts';
import { isolate, t } from '../../i18n';
import { AdminTotpRequired, createAdminTotpVerifier, type AdminTotpVerifier } from './shell/adminAuth.ts';
import * as css from './admin.css.ts';

export interface AdminLoginProps {
  /** Second-step client. Defaults to the same-origin HTTP client; tests mock it. */
  verifyTotp?: AdminTotpVerifier | undefined;
}

export function AdminLogin(props: AdminLoginProps = {}): JSX.Element {
  const admin = useAdmin();
  const verify = props.verifyTotp ?? createAdminTotpVerifier();
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [code, setCode] = createSignal('');
  const [pending, setPending] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await admin.login(username(), password());
    } catch (err) {
      if (err instanceof AdminTotpRequired) {
        setPassword('');
        setPending(err.challenge.pendingToken);
      } else {
        setError(
          err instanceof AdminApiError && err.status === 401
            ? t('admin-login-invalid')
            : err instanceof AdminApiError && err.status === 429
              ? t('admin-shell-login-throttled')
              : t('admin-login-unreachable'),
        );
      }
    } finally {
      setBusy(false);
    }
  }

  async function onSubmitCode(e: Event): Promise<void> {
    e.preventDefault();
    const token = pending();
    if (token === null) return;
    setError(null);
    setBusy(true);
    try {
      await verify(token, code().replace(/\s+/g, ''));
      await admin.loadSession();
    } catch (err) {
      setCode('');
      setError(
        err instanceof AdminApiError && err.status === 401
          ? t('admin-shell-totp-invalid')
          : err instanceof AdminApiError && err.status === 410
            ? t('admin-shell-totp-expired')
            : t('admin-login-unreachable'),
      );
    } finally {
      setBusy(false);
    }
  }

  function startOver(): void {
    setPending(null);
    setCode('');
    setError(null);
  }

  return (
    <main class={css.gate}>
      <Show
        when={pending() !== null}
        fallback={
          <form class={css.loginCard} onSubmit={(e) => void onSubmit(e)} aria-label={t('admin-login-form')}>
            <h1 class={css.heading}>{t('admin-brand')}</h1>
            <p class={css.note}>{t('admin-login-note')}</p>
            <Show when={admin.sessionEnded()}>
              <p class={css.error} role="status" data-testid="admin-session-ended">
                {t('admin-login-session-ended')}
              </p>
            </Show>
            <label class="field">
              <span>{t('admin-login-username')}</span>
              <input
                type="text"
                autocomplete="username"
                value={username()}
                onInput={(e) => setUsername(e.currentTarget.value)}
              />
            </label>
            <label class="field">
              <span>{t('admin-login-password')}</span>
              <input
                type="password"
                autocomplete="current-password"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
              />
            </label>
            <Show when={error()}>
              <p class={css.error} role="alert">
                {error()}
              </p>
            </Show>
            <button type="submit" class="btn btn--primary" disabled={busy()}>
              {busy() ? t('admin-login-signing-in') : t('admin-login-sign-in')}
            </button>
          </form>
        }
      >
        <form class={css.loginCard} onSubmit={(e) => void onSubmitCode(e)} aria-label={t('admin-shell-totp-form')}>
          <h1 class={css.heading}>{t('admin-brand')}</h1>
          <p class={css.note}>{t('admin-shell-totp-note', { name: isolate(username()) })}</p>
          <label class="field">
            <span>{t('admin-shell-totp-code')}</span>
            <input
              type="text"
              inputmode="numeric"
              autocomplete="one-time-code"
              pattern="[0-9 ]*"
              maxlength={8}
              value={code()}
              onInput={(e) => setCode(e.currentTarget.value)}
              ref={(el) => queueMicrotask(() => el.focus())}
            />
          </label>
          <Show when={error()}>
            <p class={css.error} role="alert">
              {error()}
            </p>
          </Show>
          <div class={css.loginActions}>
            <button type="submit" class="btn btn--primary" disabled={busy() || code().trim() === ''}>
              {busy() ? t('admin-login-signing-in') : t('admin-shell-totp-verify')}
            </button>
            <button type="button" class="btn btn--ghost" onClick={startOver} disabled={busy()}>
              {t('admin-shell-totp-start-over')}
            </button>
          </div>
        </form>
      </Show>
    </main>
  );
}
