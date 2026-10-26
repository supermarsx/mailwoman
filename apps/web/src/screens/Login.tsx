import { SigninChallenge } from '../components/SigninChallenge.tsx';
import { LoginTimeoutError } from '../api/loginTimeout.ts';
import { rememberedSignins, forgetSignin } from '../api/rememberedSignins.ts';
import { Icon } from '../components/Icon.tsx';
import * as rememberedCss from './rememberedSignins.css.ts';
import { createSignal, onMount, onCleanup, For, Show, type JSX } from 'solid-js';
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
import { applyBranding, brandAssetUrl } from '../branding.ts';
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

export function Login(props: { returnPath?: string } = {}): JSX.Element {
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
  const [verificationProof, setVerificationProof] = createSignal<string>();
  const [verificationReset, setVerificationReset] = createSignal(0);
  const needsVerification = () => config()?.antiBot?.mailbox === true;
  onMount(() => { void publicConfig().then(value => { setConfig(value); applyBranding(value); }).catch(() => undefined); });
  const [email, setEmail] = createSignal('');
  const [rememberedOpen, setRememberedOpen] = createSignal(false);
  let usernameInput: HTMLInputElement | undefined;
  const [remembered, setRemembered] = createSignal(rememberedSignins());
  onMount(() => { const update = (): void => { setRemembered(rememberedSignins()); }; window.addEventListener('storage', update); onCleanup(() => window.removeEventListener('storage', update)); });
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(ssoErrorReturn() ? t('auth-sso-error') : null);
  const [busy, setBusy] = createSignal(false);
  const [retryWait, setRetryWait] = createSignal(0);
  let retryTimer: ReturnType<typeof setInterval> | undefined;
  let retryUntil = 0;
  onCleanup(() => clearInterval(retryTimer));
  function waitBeforeRetry(seconds: number): void {
    if (!Number.isFinite(seconds) || seconds < 0) return;
    retryUntil = Math.max(retryUntil, Date.now() + seconds * 1000);
    clearInterval(retryTimer);
    const update = (): void => {
      setRetryWait(Math.max(0, Math.ceil((retryUntil - Date.now()) / 1000)));
      if (retryWait() === 0) clearInterval(retryTimer);
    };
    update();
    if (retryWait() > 0) retryTimer = setInterval(update, 250);
  }
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
    if (busy() || retryWait() > 0 || (needsVerification() && !verificationProof())) return;
    setError(null);
    setRefused(false);
    setReference(null);
    setBusy(true);
    waitBeforeRetry(config()?.retryCooldownSecs ?? 3);
    try {
      await app.login({ username: email().trim(), password: password(), ...(verificationProof() ? { altcha: verificationProof()! } : {}) });
    } catch (err) {
      if (err instanceof ApiError) setReference(err.requestId ?? null);
      if (err instanceof ApiError && err.status === 429) {
        waitBeforeRetry(err.retryAfterSecs ?? config()?.retryCooldownSecs ?? 3);
        setError(t('auth-login-rate-limited'));
      } else if (err instanceof ApiError && (err.code === 'signinChallengeRequired' || err.code === 'signinChallengeRejected')) {
        setError(t('auth-verification-required'));
      } else if (err instanceof LoginTimeoutError || (err instanceof ApiError && err.status === 504)) {
        setError(t('auth-login-timeout'));
      } else if (err instanceof TwoFactorRequired) {
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
      setVerificationProof(undefined);
      setVerificationReset(value => value + 1);
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
    <main class={`login ${authPage}`} data-mw-surface style={{'background-image':brandAssetUrl(config()?.loginBackgroundUrl)?`url("${brandAssetUrl(config()?.loginBackgroundUrl)}")`:undefined,'background-size':'cover','background-position':'center'}}>
      <form class={`login__card ${authCard}`} onSubmit={(e) => void onSubmit(e)} aria-label={t('auth-sign-in')}>
        <Show when={brandAssetUrl(config()?.logoUrl)}>{url=><img src={url()} alt={config()?.brandName||'Mailwoman'} style={{'max-width':'12rem','max-height':'6rem','object-fit':'contain'}}/>}</Show>
        <h1 class="login__title">{config()?.brandName || t('auth-app-name')}</h1>
        <Show when={config()?.loginNotice}>{(notice) => <p class="login__hint" dir="auto">{notice()}</p>}</Show>
        <div class={rememberedCss.field} onFocusOut={event => { if (!event.currentTarget.contains(event.relatedTarget as Node | null)) setRememberedOpen(false); }} onKeyDown={event => { if (event.key === 'Escape') { setRememberedOpen(false); usernameInput?.focus(); event.stopPropagation(); } }}>
          <label for="login-username">{t('auth-email')}</label>
          <div class={rememberedCss.inputRow}>
          <input id="login-username" ref={usernameInput} type="text" inputmode="email" required autocomplete="username" autocapitalize="none" spellcheck={false}
            aria-describedby={config()?.showDomainHint === true && config()?.usernameDomainHint ? 'login-domain-hint' : undefined}
            value={email()} onInput={event => setEmail(event.currentTarget.value)}
            onKeyDown={event => { if (event.key === 'ArrowDown' && remembered().length) { event.preventDefault(); setRememberedOpen(true); queueMicrotask(() => document.querySelector<HTMLButtonElement>('#remembered-signins button')?.focus()); } }} />
          <Show when={remembered().length}><button type="button" class={rememberedCss.toggle} disabled={busy()} aria-label={t('auth-remembered')} aria-expanded={rememberedOpen()} aria-controls="remembered-signins" onClick={() => setRememberedOpen(value => !value)}><Icon name="chevronDown" /></button></Show>
          </div>
          <Show when={rememberedOpen() && remembered().length}><div id="remembered-signins" class={rememberedCss.container}>
            <p class="login__hint">{t('auth-remembered-help')}</p>
            <ul class={rememberedCss.list}><For each={remembered()}>{address => <li class={rememberedCss.row}><button type="button" class={`btn btn--ghost ${rememberedCss.address}`} disabled={busy()} onClick={() => { setEmail(address); setPassword(''); setRememberedOpen(false); usernameInput?.focus(); }}><Icon name="mail" /><span>{address}</span></button><button type="button" class="btn btn--ghost" disabled={busy()} aria-label={t('auth-forget-address', {address})} onClick={() => { forgetSignin(address); setRemembered(rememberedSignins()); usernameInput?.focus(); }}><Icon name="delete" /></button></li>}</For></ul>
          </div></Show>
        </div>
        <Show when={config()?.showDomainHint === true ? config()?.usernameDomainHint : null}>{(domain) => <p id="login-domain-hint" class="login__hint">{t('auth-domain-hint', { domain: domain() })}</p>}</Show>
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
        <Show when={needsVerification()}><SigninChallenge surface="mailbox" reset={verificationReset()} onProof={setVerificationProof} /></Show>
        <button type="submit" class="btn btn--primary" disabled={busy() || retryWait() > 0 || (needsVerification() && !verificationProof())}>
          {busy() ? t('auth-signing-in') : retryWait() > 0 ? t('auth-retry-countdown', { seconds: retryWait() }) : t('auth-sign-in')}
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
                  href={`${ssoBeginPath(p.id, basePath())}${props.returnPath ? `?relayState=${encodeURIComponent(props.returnPath)}` : ''}`}
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
