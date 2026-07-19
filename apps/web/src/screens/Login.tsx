import { createSignal, onMount, For, Show, type JSX } from 'solid-js';
import { ApiError, TwoFactorRequired, type LoginChallenge } from '../api/client.ts';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';
import { listSsoProviders, ssoBeginPath, type SsoProviderSummary } from '../modules/sso';
import { TwoFactorChallenge } from './Settings/index.ts';

/**
 * Did the browser land back here from a failed SSO round-trip? The IdP
 * callback/ACS redirects to `/?sso_error=…` on failure (success sets the
 * session cookie and drops the browser straight into the inbox via `app.init`,
 * so there is no success param to read here). The value is ignored — a UNIFORM
 * message is shown, mirroring e0's no-leak 401 contract (never reveal which
 * check failed). Absent SSO, `location` has no such param and nothing renders.
 */
function ssoErrorReturn(): boolean {
  if (typeof location === 'undefined') return false;
  return new URLSearchParams(location.search).has('sso_error');
}

export function Login(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('auth'));
  onMount(() => void loadCatalog('login2fa'));
  const [jmapUrl, setJmapUrl] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(ssoErrorReturn() ? t('auth-sso-error') : null);
  const [busy, setBusy] = createSignal(false);

  // When the password is accepted but a second factor is required, the login is
  // NOT complete: `app.login` threw `TwoFactorRequired` before any session was
  // established. Hold the challenge and render `<TwoFactorChallenge>`; only once a
  // factor verifies (its `onSuccess`) do we bootstrap the session — no downgrade.
  const [challenge, setChallenge] = createSignal<LoginChallenge | null>(null);

  async function onFactorCleared(): Promise<void> {
    // The factor cleared and the server issued the session cookie; run the same
    // post-login bootstrap the normal path does (`app.init` reads `/api/me`).
    await app.init();
  }

  // Advertise configured IdPs (pre-auth). Fail-soft to `[]` — a deployment with
  // no SSO renders the login exactly as today (the `<Show>` blocks collapse).
  const [providers, setProviders] = createSignal<SsoProviderSummary[]>([]);
  onMount(() => void listSsoProviders().then(setProviders).catch(() => setProviders([])));

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await app.login({ jmapUrl: jmapUrl(), username: username(), password: password() });
    } catch (err) {
      if (err instanceof TwoFactorRequired) {
        // Not an error: swap the credential form for the second-factor challenge.
        setChallenge(err.challenge);
      } else if (err instanceof ApiError && err.status === 401) {
        setError(t('auth-invalid-credentials'));
      } else {
        setError(t('auth-unreachable'));
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <Show when={challenge()} fallback={credentialForm()}>
      {(ch) => (
        <main class="login">
          <div class="login__card">
            <h1 class="login__title">{t('auth-app-name')}</h1>
            <Show
              when={!ch().enrollmentRequired}
              fallback={
                <p class="login__hint" role="status" data-testid="twofa-enroll-required">
                  {t('login-2fa-enroll-required')}
                </p>
              }
            >
              <TwoFactorChallenge challenge={ch()} onSuccess={() => void onFactorCleared()} />
            </Show>
            <button type="button" class="btn btn--ghost" onClick={() => setChallenge(null)}>
              {t('login-2fa-back')}
            </button>
          </div>
        </main>
      )}
    </Show>
  );

  function credentialForm(): JSX.Element {
    return (
    <main class="login">
      <form class="login__card" onSubmit={(e) => void onSubmit(e)} aria-label={t('auth-sign-in')}>
        <h1 class="login__title">{t('auth-app-name')}</h1>
        <label class="field">
          <span>{t('auth-jmap-url')}</span>
          <input
            type="url"
            required
            placeholder={t('auth-jmap-url-placeholder')}
            value={jmapUrl()}
            onInput={(e) => setJmapUrl(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>{t('auth-username')}</span>
          <input
            type="text"
            required
            autocomplete="username"
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>{t('auth-password')}</span>
          <input
            type="password"
            required
            autocomplete="current-password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
          />
        </label>
        <Show when={error()}>
          <p class="login__error" role="alert">
            {error()}
          </p>
        </Show>
        <button type="submit" class="btn btn--primary" disabled={busy()}>
          {busy() ? t('auth-signing-in') : t('auth-sign-in')}
        </button>
        <Show when={providers().length > 0}>
          <div class="login__sso" role="group" aria-label={t('auth-sso-heading')}>
            <p class="login__sso-divider" aria-hidden="true">
              {t('auth-sso-divider')}
            </p>
            <For each={providers()}>
              {(p) => (
                <a
                  class="btn btn--ghost login__sso-btn"
                  href={ssoBeginPath(p.id)}
                  data-sso-id={p.id}
                  rel="nofollow"
                >
                  {t('auth-sso-button', { name: p.displayName })}
                </a>
              )}
            </For>
          </div>
        </Show>
        <p class="login__hint">{t('auth-mock-hint')}</p>
      </form>
    </main>
    );
  }
}
