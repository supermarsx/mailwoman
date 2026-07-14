import { createSignal, onMount, For, Show, type JSX } from 'solid-js';
import { ApiError } from '../api/client.ts';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';
import { listSsoProviders, ssoBeginPath, type SsoProviderSummary } from '../modules/sso';

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
  const [jmapUrl, setJmapUrl] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(ssoErrorReturn() ? t('auth-sso-error') : null);
  const [busy, setBusy] = createSignal(false);

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
      if (err instanceof ApiError && err.status === 401) {
        setError(t('auth-invalid-credentials'));
      } else {
        setError(t('auth-unreachable'));
      }
    } finally {
      setBusy(false);
    }
  }

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
