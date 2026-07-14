// Admin sign-in gate (plan §2.5 — SEPARATE session domain). The panel is gated on
// an admin session distinct from the mailbox cookie; when none is present this
// form authenticates against `/admin/login`. e11 backs it with the admin session
// domain (passkey-capable); this is the password fallback surface.

import { createSignal, Show, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import { AdminApiError } from '../../state/slices/admin.ts';
import { t } from '../../i18n';
import * as css from './admin.css.ts';

export function AdminLogin(): JSX.Element {
  const admin = useAdmin();
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await admin.login(username(), password());
    } catch (err) {
      setError(
        err instanceof AdminApiError && err.status === 401
          ? t('admin-login-invalid')
          : t('admin-login-unreachable'),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <main class={css.gate}>
      <form class={css.card} onSubmit={(e) => void onSubmit(e)} aria-label={t('admin-login-form')}>
        <h1 class={css.heading}>{t('admin-brand')}</h1>
        <p class={css.note}>{t('admin-login-note')}</p>
        <label class="field">
          <span>{t('admin-login-username')}</span>
          <input type="text" autocomplete="username" value={username()} onInput={(e) => setUsername(e.currentTarget.value)} />
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
    </main>
  );
}
