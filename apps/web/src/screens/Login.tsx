import { createSignal, onMount, For, Show, type JSX } from 'solid-js';
import {
  ApiError,
  TwoFactorRequired,
  type DiscoveredServer,
  type DiscoverResult,
  type LoginChallenge,
  type LoginInput,
} from '../api/client.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { basePath } from '../api/basePath.ts';
import { useApp } from '../state/context.ts';
import { t, loadCatalog, isolate } from '../i18n';
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

/** A lookup result reduced to what the sign-in will send. */
interface FoundServer {
  /** The part of the typed address after the last `@`. */
  domain: string;
  /** The value sent as `jmapUrl` to `/api/login`. */
  url: string;
  /** The IMAP endpoint behind `url`; `null` when `url` is a JMAP session URL. */
  imap: DiscoveredServer | null;
  source: DiscoverResult['source'];
  /** The lookup says the provider expects OAuth2, which this screen cannot do. */
  oauthOnly: boolean;
}

/** A DNS host name: letters, digits, dots and hyphens only. */
function isHostName(host: unknown): host is string {
  return typeof host === 'string' && /^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$/i.test(host);
}

function isPort(port: unknown): port is number {
  return typeof port === 'number' && Number.isInteger(port) && port >= 1 && port <= 65535;
}

/**
 * Turn a `/api/discover` answer into the server URL `/api/login` reads from its
 * `jmapUrl` field, or `null` when the answer names no server this screen can
 * use.
 *
 * - `source: "jmap"`: the lookup fetched `https://<domain>/.well-known/jmap`
 *   and got a session resource (`crates/mw-autoconfig/src/lib.rs:176-180`), so
 *   that URL is the one to sign in with. The `imap` entry of such an answer is
 *   the JMAP API host, not an IMAP server, and is ignored.
 * - `source: "jmap-srv"`: only a `_jmap._tcp` SRV record was found; the session
 *   resource is at `/.well-known/jmap` on that host and port (RFC 8620 §2.2).
 * - anything else: the IMAP endpoint, as `imaps://host:port` for TLS from
 *   connect and `imap://host:port` otherwise — the two IMAP spellings the
 *   server parses (`crates/mw-server/src/engine_mode.rs:51-83`).
 *
 * The server does not say whether it proxies a JMAP upstream or drives an IMAP
 * account itself, so a URL built here can be of the kind this deployment does
 * not accept. That sign-in is refused like a wrong password; `signIn` then
 * opens the manual fields with what was sent.
 */
function planSignIn(email: string, result: DiscoverResult): FoundServer | null {
  const domain = email.slice(email.lastIndexOf('@') + 1);
  const base = { domain, source: result.source, oauthOnly: result.auth === 'oauth2' };
  if (result.source === 'jmap') {
    if (!isHostName(domain)) return null;
    return { ...base, url: `https://${domain}/.well-known/jmap`, imap: null };
  }
  if (result.source === 'jmap-srv') {
    const srv = result.jmapSrv;
    if (srv === undefined || !isHostName(srv.host) || !isPort(srv.port)) return null;
    const authority = srv.port === 443 ? srv.host : `${srv.host}:${srv.port}`;
    return { ...base, url: `https://${authority}/.well-known/jmap`, imap: null };
  }
  const imap = result.imap;
  if (imap === undefined || !isHostName(imap.host) || !isPort(imap.port)) return null;
  const scheme = imap.tls === 'implicit' ? 'imaps' : 'imap';
  return { ...base, url: `${scheme}://${imap.host}:${imap.port}`, imap };
}

export function Login(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('auth'));
  onMount(() => void loadCatalog('login2fa'));
  const [email, setEmail] = createSignal('');
  // The server the last lookup found for `email()`, shown for confirmation.
  // Cleared whenever the address is edited, so it never describes another one.
  const [found, setFound] = createSignal<FoundServer | null>(null);
  // The manual fields (server URL, username) replace the email field.
  const [manual, setManual] = createSignal(false);
  // Set when a sign-in that used a looked-up server was refused (401).
  const [foundRefused, setFoundRefused] = createSignal<FoundServer | null>(null);
  let emailInput: HTMLInputElement | undefined;
  let urlInput: HTMLInputElement | undefined;
  const [jmapUrl, setJmapUrl] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(ssoErrorReturn() ? t('auth-sso-error') : null);
  const [busy, setBusy] = createSignal(false);
  // Set when the server refused the credentials (401). A disabled account gets
  // the same 401 as a wrong password — the server keeps one shape so the refusal
  // does not reveal account state — so the form cannot say WHICH it was; it adds
  // a note naming the other possibility.
  const [refused, setRefused] = createSignal(false);

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
  onMount(() => void listSsoProviders(basePath()).then(setProviders).catch(() => setProviders([])));

  function showManual(): void {
    setManual(true);
    if (username() === '') setUsername(email().trim());
    urlInput?.focus();
  }

  function showLookup(): void {
    setManual(false);
    setFound(null);
    setError(null);
    setRefused(false);
    setFoundRefused(null);
    emailInput?.focus();
  }

  /** The lookup gave nothing to sign in with: open the manual fields, keeping the address. */
  function fallBackToManual(address: string, message: string): void {
    setError(message);
    setUsername(address);
    setManual(true);
    urlInput?.focus();
  }

  async function lookUp(): Promise<void> {
    const address = email().trim();
    const domain = isolate(address.slice(address.lastIndexOf('@') + 1));
    setBusy(true);
    try {
      const result = await createConfiguredClient().discover?.(address);
      const plan = result === undefined ? null : planSignIn(address, result);
      if (plan === null) fallBackToManual(address, t('auth-discover-failed'));
      else setFound(plan);
    } catch (err) {
      const status = err instanceof ApiError ? err.status : null;
      if (status === 404) fallBackToManual(address, t('auth-discover-not-found', { domain }));
      else if (status === 400) fallBackToManual(address, t('auth-discover-invalid-email', { email: isolate(address) }));
      else if (status === 429) fallBackToManual(address, t('auth-discover-rate-limited'));
      else fallBackToManual(address, t('auth-discover-failed'));
    } finally {
      setBusy(false);
    }
  }

  /** Sign in. `via` is the looked-up server `input` was built from, if any. */
  async function signIn(input: LoginInput, via: FoundServer | null): Promise<void> {
    setBusy(true);
    try {
      await app.login(input);
    } catch (err) {
      if (err instanceof TwoFactorRequired) {
        // Not an error: swap the credential form for the second-factor challenge.
        setChallenge(err.challenge);
      } else if (err instanceof ApiError && err.status === 401) {
        setError(t('auth-invalid-credentials'));
        setRefused(true);
        if (via !== null) {
          // The refusal does not say whether the server, the username or the
          // password was wrong, so put what was sent where it can be changed.
          setFoundRefused(via);
          setJmapUrl(input.jmapUrl);
          setUsername(input.username);
          setFound(null);
          setManual(true);
          urlInput?.focus();
        }
      } else {
        setError(t('auth-unreachable'));
      }
    } finally {
      setBusy(false);
    }
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setRefused(false);
    setFoundRefused(null);
    if (manual()) {
      await signIn({ jmapUrl: jmapUrl(), username: username(), password: password() }, null);
      return;
    }
    const server = found();
    if (server === null) {
      await lookUp();
    } else if (!server.oauthOnly) {
      await signIn({ jmapUrl: server.url, username: email().trim(), password: password() }, server);
    }
  }

  function submitLabel(): string {
    if (!manual() && found() === null) return busy() ? t('auth-discovering') : t('auth-sign-in');
    if (busy()) return t('auth-signing-in');
    return manual() ? t('auth-sign-in') : t('auth-discover-confirm');
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
        <Show
          when={manual()}
          fallback={
            <label class="field">
              <span>{t('auth-email')}</span>
              <input
                ref={emailInput}
                type="email"
                required
                autocomplete="username"
                value={email()}
                onInput={(e) => {
                  setEmail(e.currentTarget.value);
                  setFound(null);
                }}
              />
            </label>
          }
        >
          <label class="field">
            <span>{t('auth-jmap-url')}</span>
            <input
              ref={urlInput}
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
        </Show>
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
        <Show when={refused()}>
          <p class="login__hint" data-testid="login-refused-note">
            {t('auth-refused-note')}
          </p>
        </Show>
        <Show when={manual() ? foundRefused() : null}>
          {(server) => (
            <p class="login__hint" data-testid="login-discovered-refused-note">
              {t('auth-discover-refused-note', { url: isolate(server().url), domain: isolate(server().domain) })}
            </p>
          )}
        </Show>
        {/* Always in the document, so a result that arrives later is announced. */}
        <div role="status" data-testid="login-discovered">
          <Show when={manual() ? null : found()}>
            {(server) => (
              <>
                <p class="login__hint">
                  <Show
                    when={server().imap}
                    fallback={t('auth-discover-found-jmap', {
                      domain: isolate(server().domain),
                      url: isolate(server().url),
                    })}
                  >
                    {(imap) =>
                      t('auth-discover-found-imap', {
                        domain: isolate(server().domain),
                        host: isolate(imap().host),
                        port: String(imap().port),
                        tls: imap().tls,
                      })
                    }
                  </Show>
                </p>
                <p class="login__hint">{t('auth-discover-source', { source: server().source })}</p>
                <Show when={server().oauthOnly}>
                  <p class="login__hint" data-testid="login-oauth-only">
                    {t('auth-discover-oauth-only')}
                  </p>
                </Show>
              </>
            )}
          </Show>
        </div>
        <Show when={manual() || found()?.oauthOnly !== true}>
          <button type="submit" class="btn btn--primary" disabled={busy()}>
            {submitLabel()}
          </button>
        </Show>
        <button type="button" class="btn btn--ghost" onClick={() => (manual() ? showLookup() : showManual())}>
          {manual() ? t('auth-manual-hide') : t('auth-manual-show')}
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
