import { SigninChallenge } from '../../components/SigninChallenge.tsx';
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

import { Show, createSignal, onMount, onCleanup, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import { AdminApiError } from '../../state/slices/admin.ts';
import { isolate, t, loadCatalog } from '../../i18n';
import { AdminTotpRequired, createAdminTotpVerifier, signInAdminPasskey, completeAdminRecovery, type AdminTotpVerifier } from './shell/adminAuth.ts';
import { passkeySupported } from '../Settings/webauthn.ts';
import * as css from './admin.css.ts';
import * as brandCss from './branding.css.ts';
import { authCard, authPage } from '../../styles/authCard.css.ts';
import { PasswordInput } from '../../components/PasswordInput.tsx';
import {useBranding,brandAssetUrl} from '../../branding.ts';

export interface AdminLoginProps {
  /** Second-step client. Defaults to the same-origin HTTP client; tests mock it. */
  verifyTotp?: AdminTotpVerifier | undefined;
}

export function AdminLogin(props: AdminLoginProps = {}): JSX.Element {
  const branding=useBranding();
  const [verificationProof, setVerificationProof] = createSignal<string>();
  const [verificationReset, setVerificationReset] = createSignal(0);
  const needsVerification = () => branding()?.antiBot?.admin === true;
  const admin = useAdmin();
  const verify = props.verifyTotp ?? createAdminTotpVerifier();
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [code, setCode] = createSignal('');
  const [pending, setPending] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [retrySeconds, setRetrySeconds] = createSignal(0);
  let retryTimer: ReturnType<typeof setInterval> | undefined;
  function waitForRetry(seconds: number): void {
    if (retryTimer !== undefined) clearInterval(retryTimer);
    const deadline = Date.now() + seconds * 1000;
    setRetrySeconds(seconds);
    retryTimer = setInterval(() => {
      setRetrySeconds(Math.max(0, Math.ceil((deadline - Date.now()) / 1000)));
      if (retrySeconds() === 0) { clearInterval(retryTimer); retryTimer = undefined; }
    }, 250);
  }
  onCleanup(() => { if (retryTimer !== undefined) clearInterval(retryTimer); });
  const [factors,setFactors]=createSignal<string[]>([]);
  const [recovery,setRecovery]=createSignal(false);
  onMount(()=>void loadCatalog('admin-passkeys'));
  async function passkeyLogin():Promise<void>{setBusy(true);setError(null);try{await signInAdminPasskey();await admin.loadSession();}catch{setError(t('admin-passkeys-failed'));}finally{setBusy(false);}}

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    if (busy() || retrySeconds() > 0 || (needsVerification() && !verificationProof())) return;
    setError(null);
    setBusy(true);
    try {
      await admin.login(username(), password(), verificationProof());
    } catch (err) {
      if (err instanceof AdminTotpRequired) {
        setPassword('');
        setPending(err.challenge.pendingToken);
        setFactors(err.challenge.factors);
        setRecovery(err.challenge.factors.includes('recovery') && !err.challenge.factors.includes('totp'));
      } else {
        if (err instanceof AdminApiError && err.status === 429) waitForRetry(err.retryAfterSecs ?? 3);
        setError(
          err instanceof AdminApiError && err.status === 403 && needsVerification()
            ? t('auth-verification-required')
            : err instanceof AdminApiError && err.status === 401
            ? t('admin-login-invalid')
            : err instanceof AdminApiError && err.status === 429
              ? t('admin-shell-login-throttled')
              : t('admin-login-unreachable'),
        );
      }
    } finally {
      setBusy(false);
      setVerificationProof(undefined);
      setVerificationReset(value => value + 1);
    }
  }

  async function onSubmitCode(e: Event): Promise<void> {
    e.preventDefault();
    const token = pending();
    if (token === null) return;
    setError(null);
    setBusy(true);
    try {
      if(recovery()) await completeAdminRecovery(token,code().trim());
      else await verify(token, code().replace(/\s+/g, ''));
      await admin.loadSession();
    } catch (err) {
      setCode('');
      if(recovery()){setPending(null);setError(t('admin-passkeys-recovery-failed'));return;}
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
    <main class={`${css.gate} ${authPage} ${brandCss.loginBackdropHost}`}>
      <Show when={brandAssetUrl(branding()?.loginBackgroundUrl)}>{url=><img class={brandCss.loginBackdrop} src={url()} alt="" aria-hidden="true"/>}</Show>
      <Show
        when={pending() !== null}
        fallback={
          <form class={`${css.loginCard} ${authCard}`} onSubmit={(e) => void onSubmit(e)} aria-label={t('admin-login-form')}>
            <Show when={brandAssetUrl(branding()?.logoUrl)}>{url=><img src={url()} alt={branding()?.brandName||'Mailwoman'} class={brandCss.loginLogo}/>}</Show>
            <h1 class={css.heading}>{branding()?.brandName||t('admin-brand')}</h1>
            <p class={css.note}>{t('admin-login-note')}</p>
            <Show when={branding()?.loginNotice}>{notice=><p class={css.note} dir="auto">{notice()}</p>}</Show>
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
              <PasswordInput
                label={t('admin-login-password')}
                autocomplete="current-password"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
                hideWhen={error}
              />
            </label>
            <Show when={error()}>
              <p class={css.error} role="alert">
                {error()}
              </p>
            </Show>
            <Show when={retrySeconds() > 0}><p role="status">{t("admin-login-retry-countdown", { seconds: retrySeconds() })}</p></Show>
            <Show when={needsVerification()}><SigninChallenge surface="admin" reset={verificationReset()} onProof={setVerificationProof} /></Show>
            <button type="submit" class="btn btn--primary" disabled={busy() || retrySeconds() > 0 || (needsVerification() && !verificationProof())}>
              {busy() ? t('admin-login-signing-in') : t('admin-login-sign-in')}
            </button>
            <Show when={passkeySupported()}><button type="button" class="btn" disabled={busy()} onClick={()=>void passkeyLogin()}>{t('admin-passkeys-sign-in')}</button></Show>
          </form>
        }
      >
        <form class={`${css.loginCard} ${authCard}`} onSubmit={(e) => void onSubmitCode(e)} aria-label={t('admin-shell-totp-form')}>
          <h1 class={css.heading}>{branding()?.brandName||t('admin-brand')}</h1>
          <p class={css.note}>{factors().includes('totp') ? t('admin-shell-totp-note', { name: isolate(username()) }) : t('admin-passkeys-factor-note')}</p>
          <Show when={factors().includes('webauthn') && passkeySupported()}><button class="btn" type="button" disabled={busy()} onClick={()=>void passkeyLogin()}>{t('admin-passkeys-use')}</button></Show>
          <Show when={factors().includes('recovery')}><label class="field"><input type="checkbox" checked={recovery()} onChange={e=>{setRecovery(e.currentTarget.checked);setCode('');}}/>{t('admin-passkeys-recovery-use')}</label></Show>
          <Show when={recovery() || factors().includes('totp')}>
          <label class="field">
            <span>{t(recovery()?'admin-passkeys-recovery-code':'admin-shell-totp-code')}</span>
            <input
              type="text"
              inputmode={recovery()?'text':'numeric'}
              autocomplete="one-time-code"
              pattern={recovery()?undefined:'[0-9 ]*'}
              maxlength={recovery()?128:8}
              value={code()}
              onInput={(e) => setCode(e.currentTarget.value)}
              ref={(el) => queueMicrotask(() => el.focus())}
            />
          </label>
          </Show>
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
