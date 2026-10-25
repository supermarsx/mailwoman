import { createSignal, onMount, For, Show, type JSX } from 'solid-js';
import { ApiError, TwoFactorRequired, type LoginChallenge } from '../api/client.ts';
import { isNativeAuth } from '../api/serverAddress.ts';
import { basePath } from '../api/basePath.ts';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';
import { listSsoProviders, ssoBeginPath, type SsoProviderSummary } from '../modules/sso';
import { TwoFactorChallenge } from './Settings/index.ts';
import { authCard, authPage } from '../styles/authCard.css.ts';
import { PasswordInput } from '../components/PasswordInput.tsx';
import { publicConfig, type PublicConfig } from '../api/publicConfig.ts';
import { passwordlessAvailable, signInWithPasskey } from '../api/passwordless.ts';

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
  // A desktop or mobile shell talks to the server it was built or provisioned for
  // (t32); the user never enters a server address. Nor does a browser (t32-h9):
  // the server decides which mail server an address signs in to.
  const native = isNativeAuth();
  const [passkeyAvailable, setPasskeyAvailable] = createSignal(false);
  onMount(() => { if (!native) void passwordlessAvailable().then(setPasskeyAvailable).catch(() => undefined); });
  const [config, setConfig] = createSignal<PublicConfig | null>(null);
  onMount(() => { void publicConfig().then(setConfig).catch(() => undefined); });
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(ssoErrorReturn() ? t('auth-sso-error') : null);
  const [busy, setBusy] = createSignal(false);
  // Set when the server refused the credentials (401). A disabled account gets
  // the same 401 as a wrong password — the server keeps one shape so the refusal
  // does not reveal account state — so the form cannot say WHICH it was; it adds
  // a note naming the other possibility.
  const [refused, setRefused] = createSignal(false);
  // The id of the server's answer to a failed sign-in, when it sent one
  // (`X-Request-Id`), so a support request can name it.
  const [reference, setReference] = createSignal<string | null>(null);

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
  // Not in a shell: an IdP round-trip lands on the server's own pages, which the
  // shell does not load, so the shells sign in with a password only.
  onMount(() => {
    if (native) return;
    void listSsoProviders(basePath()).then(setProviders).catch(() => setProviders([]));
  });

  async function passkeySignIn(): Promise<void> {
    setBusy(true); setError(null); setRefused(false); setReference(null);
    try { await signInWithPasskey(native); await app.init(); }
    catch { setError(t('auth-passkey-failed')); }
    finally { setBusy(false); }
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setRefused(false);
    setReference(null);
    setBusy(true);
    try {
      await app.login({ username: email().trim(), password: password() });
    } catch (err) {
      if (err instanceof ApiError) setReference(err.requestId ?? null);
      if (err instanceof TwoFactorRequired) {
        // Not an error: swap the credential form for the second-factor challenge.
        setChallenge(err.challenge);
      } else if (err instanceof ApiError && err.status === 401) {
        setError(t('auth-invalid-credentials'));
        setRefused(true);
      } else if (err instanceof ApiError && err.status === 403 && err.code === 'accountDisabled') {
        // The administrator chose to explain refusals (`signin.refusalDetail`).
        setError(t('auth-account-disabled'));
      } else if (err instanceof ApiError && err.status === 403 && err.code === 'signinNotAllowed') {
        setError(t('auth-signin-not-allowed'));
      } else if (err instanceof ApiError && err.status === 404) {
        // No upstream the operator configured places this address.
        setError(t('auth-address-not-configured'));
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
        <main class={`login ${authPage}`}>
          <div class={`login__card ${authCard}`}>
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
    <main class={`login ${authPage}`} data-mw-surface>
      <form class={`login__card ${authCard}`} onSubmit={(e) => void onSubmit(e)} aria-label={t('auth-sign-in')}>
        <h1 class="login__title">{config()?.brandName || t('auth-app-name')}</h1>
        <Show when={config()?.loginNotice}>{(notice) => <p class="login__hint" dir="auto">{notice()}</p>}</Show>
        <label class="field">
          <span>{t('auth-email')}</span>
          {/* Text, not `type=email`: a deployment may sign in with a bare user name. */}
          <input
            type="text"
            inputmode="email"
            required
            autocomplete="username"
            autocapitalize="none"
            spellcheck={false}
            aria-describedby={config()?.usernameDomainHint ? 'login-domain-hint' : undefined}
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
          />
        </label>
        <Show when={config()?.usernameDomainHint}>{(domain) => <p id="login-domain-hint" class="login__hint">{t('auth-domain-hint', { domain: domain() })}</p>}</Show>
        <label class="field">
          <span>{t('auth-password')}</span>
          <PasswordInput
            label={t('auth-password')}
            required
            autocomplete="current-password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
            hideWhen={error}
          />
        </label>
        <Show when={error()}>
          <p class="login__error" role="alert">
            {error()}
          </p>
        </Show>
        <Show when={refused()}>
          <p class="login__hint" data-testid="login-refused-note">
            {t('auth-refused-note')}
          </p>
        </Show>
        <Show when={error() !== null ? reference() : null}>
          {(id) => (
            <p class="login__hint" data-testid="login-reference">
              {t('auth-reference', { id: id() })}
            </p>
          )}
        </Show>
        <button type="submit" class="btn btn--primary" disabled={busy()}>
          {busy() ? t('auth-signing-in') : t('auth-sign-in')}
        </button>
        <Show when={passkeyAvailable()}>
          <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => void passkeySignIn()}>{t('auth-passkey-sign-in')}</button>
        </Show>
        <Show when={providers().length > 0}>
          <div class="login__sso" role="group" aria-label={t('auth-sso-heading')}>
            <p class="login__sso-divider" aria-hidden="true">
              {t('auth-sso-divider')}
            </p>
            <For each={providers()}>
              {(p) => (
                <a
                  class="btn btn--ghost login__sso-btn"
                  href={ssoBeginPath(p.id, basePath())}
                  data-sso-id={p.id}
                  rel="nofollow"
                >
                  {t('auth-sso-button', { name: p.displayName })}
                </a>
              )}
            </For>
          </div>
        </Show>
      </form>
    </main>
    );
  }
}
