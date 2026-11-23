// Forced second-factor enrolment on the sign-in screen (t47-e2).
//
// `/api/login` answered `twofaRequired` + `enrollmentRequired`: the password is
// right, an administrator requires a second factor, and the account has none.
// No session exists yet. This step enrols one over the pending token (an
// authenticator app, or a passkey where the server offers it and this client can
// make one) and the confirming call issues the session and the account's first
// recovery codes, which are shown once and must be acknowledged before the app
// opens. Server: `login_enroll_*` in `crates/mw-server/src/twofa_routes.rs`.
//
// The pieces are the settings enrolment's (`Settings/TwoFactor.tsx`): its
// strings, its passkey ceremony, its recovery-code panel. Nothing here keeps the
// secret or the codes anywhere but the component's own signals.

import { createMemo, createSignal, createEffect, For, Show, onCleanup, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../i18n';
import { ApiError } from '../api/errors.ts';
import { loginEnrolmentApi, type LoginEnrolmentApi, type LoginTotpSetup } from '../api/loginEnrolment.ts';
import type { LoginChallenge } from '../api/client.ts';
import { encodeQr, qrToSvg } from '../modules/keys/qr.ts';
import { passkeySupported, registerPasskey } from './Settings/webauthn.ts';
import { PASSKEY_POLICY_CODES } from './Settings/service.ts';
import * as css from './Settings/styles.css.ts';
import * as own from './loginEnrolment.css.ts';

/**
 * How long the server keeps a pending sign-in (`twofa.pendingTtlSecs`, default
 * 300 s). The server does not send it, and answers a wrong code and an expired
 * sign-in with the same 401, so a refusal after this long is read as expired.
 */
export const PENDING_TTL_SECS = 300;

export type EnrolMethod = 'totp' | 'webauthn';

/** The factors this client can enrol from the server's offer, in display order. */
export function enrolMethods(challenge: LoginChallenge, allowPasskey: boolean): EnrolMethod[] {
  const out: EnrolMethod[] = [];
  if (challenge.factors.includes('totp')) out.push('totp');
  if (allowPasskey && challenge.factors.includes('webauthn') && passkeySupported()) out.push('webauthn');
  return out;
}

export interface LoginEnrolmentProps {
  challenge: LoginChallenge;
  /** Passkeys may be offered (false in the native shells, as for passkey sign-in). */
  allowPasskey: boolean;
  /** The session was issued and the codes acknowledged: bootstrap the app. */
  onComplete: () => void | Promise<void>;
  /** The pending sign-in is gone: return to the credential form and say why. */
  onExpired: () => void;
  /** Return to the credential form (before anything was enrolled). */
  onBack: () => void;
  api?: LoginEnrolmentApi;
  /** Injected in tests. */
  now?: () => number;
}

type Step = 'begin' | 'confirm' | 'passkey';

type Phase =
  | { kind: 'choose' }
  | { kind: 'totp'; setup: LoginTotpSetup }
  | { kind: 'passkey' }
  | { kind: 'codes'; codes: readonly string[] };

/** The browser refused the registration ceremony (cancelled, no authenticator). */
function browserRefused(e: unknown): boolean {
  return e instanceof Error && (e.name === 'NotAllowedError' || e.name === 'NotSupportedError' || e.name === 'ConstraintError' || e.name === 'InvalidStateError' || e.name === 'AbortError');
}

export function LoginEnrolment(props: LoginEnrolmentProps): JSX.Element {
  const api = props.api ?? loginEnrolmentApi;
  const now = props.now ?? Date.now;
  const started = now();
  onMount(() => {
    void loadCatalog('settings');
    void loadCatalog('login2fa');
    void loadCatalog('auth');
  });

  const methods = createMemo(() => enrolMethods(props.challenge, props.allowPasskey));
  const [phase, setPhase] = createSignal<Phase>({ kind: 'choose' });
  const [code, setCode] = createSignal('');
  const [label, setLabel] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  let heading: HTMLHeadingElement | undefined;

  // Move focus to the step's heading whenever the step changes, so a screen
  // reader announces it and keyboard users start at its top.
  createEffect(() => {
    phase();
    queueMicrotask(() => heading?.focus());
  });

  // While the one-time codes are on screen the session already exists: leaving
  // the page would lose them, so ask first (they can be replaced in Settings).
  createEffect(() => {
    if (phase().kind !== 'codes') return;
    const guard = (event: BeforeUnloadEvent): void => { event.preventDefault(); };
    window.addEventListener('beforeunload', guard);
    onCleanup(() => window.removeEventListener('beforeunload', guard));
  });

  const expired = (): boolean => now() - started >= PENDING_TTL_SECS * 1000;

  /** Show a refusal, or leave for the credential form when the sign-in is gone. */
  function fail(e: unknown, step: Step): void {
    if (e instanceof ApiError) {
      if (e.status === 401) {
        // A begin has no code to get wrong: its 401 is a pending sign-in that is
        // gone. A confirm's 401 is a wrong code unless the sign-in has timed out.
        if (step === 'begin' || expired()) { props.onExpired(); return; }
        setError(step === 'confirm' ? t('login-2fa-enrol-code-rejected') : t('settings-2fa-error-passkey'));
        return;
      }
      if (e.status === 429) { setError(t('auth-login-rate-limited')); return; }
      const policy = e.code !== undefined ? PASSKEY_POLICY_CODES[e.code] : undefined;
      if (policy !== undefined) { setError(t(policy)); return; }
      if (e.code === 'passkeysDisabled') { setError(t('settings-2fa-passkey-policy-disabled')); return; }
      setError(t('settings-2fa-error-generic'));
      return;
    }
    if (browserRefused(e)) { setError(t('settings-2fa-error-passkey')); return; }
    // Not an answer from the server: it was not reached.
    setError(t('auth-unreachable'));
  }

  async function run(action: () => Promise<void>, step: Step | (() => Step)): Promise<void> {
    if (busy()) return;
    setBusy(true);
    setError('');
    try { await action(); } catch (e) { fail(e, typeof step === 'function' ? step() : step); } finally { setBusy(false); }
  }

  function enrolled(codes: readonly string[]): void {
    setCode('');
    // The session is issued. An account that already had recovery codes gets
    // none back; there is nothing to show, so the app opens.
    if (codes.length === 0) { void props.onComplete(); return; }
    setSaved(false);
    setPhase({ kind: 'codes', codes });
  }

  const beginTotp = (): Promise<void> => run(async () => {
    const setup = await api.totpBegin(props.challenge.pendingToken);
    setCode('');
    setPhase({ kind: 'totp', setup });
  }, 'begin');

  const confirmTotp = (event: Event): Promise<void> => {
    event.preventDefault();
    const value = code().replace(/\s+/g, '');
    if (value === '') return Promise.resolve();
    return run(async () => enrolled((await api.totpConfirm(props.challenge.pendingToken, value)).recoveryCodes), 'confirm');
  };

  const enrolPasskey = (): Promise<void> => {
    // A refused begin means the sign-in is gone; a refused finish may be either.
    let step: Step = 'begin';
    return run(async () => {
    const challenge = await api.passkeyBegin(props.challenge.pendingToken);
    step = 'passkey';
    const ceremony = await registerPasskey(challenge);
    const out = await api.passkeyFinish(props.challenge.pendingToken, {
      clientDataJson: ceremony.clientDataJson,
      attestationObject: ceremony.attestationObject,
      transports: ceremony.transports,
      label: label().trim(),
      ...(ceremony.authenticatorAttachment !== undefined ? { authenticatorAttachment: ceremony.authenticatorAttachment } : {}),
      ...(ceremony.residentKey !== undefined ? { residentKey: ceremony.residentKey } : {}),
    });
    enrolled(out.recoveryCodes);
    }, () => step);
  };

  const qr = createMemo(() => {
    const current = phase();
    if (current.kind !== 'totp') return undefined;
    try { return `data:image/svg+xml,${encodeURIComponent(qrToSvg(encodeQr(current.setup.otpauthUri)))}`; } catch { return undefined; }
  });

  const title = (): string => (phase().kind === 'codes' ? t('settings-2fa-recovery-title') : t('login-2fa-enrol-title'));
  const canGoBack = (): boolean => phase().kind !== 'codes';

  return (
    <section class={css.section} aria-labelledby="login-enrol-title" data-testid="twofa-enrol">
      <h2 class={css.heading} id="login-enrol-title" tabindex="-1" ref={heading}>{title()}</h2>

      <Show when={phase().kind === 'choose'}>
        <p class={css.prose}>{t('login-2fa-enrol-intro')}</p>
        <div class={css.actions} role="group" aria-label={t('login-2fa-enrol-choose')}>
          <For each={methods()}>
            {(method) => method === 'totp'
              ? <button type="button" class={css.button} disabled={busy()} onClick={() => void beginTotp()} data-testid="enrol-totp">{t('settings-2fa-totp-enrol')}</button>
              : <button type="button" class={methods().length > 1 ? css.ghost : css.button} disabled={busy()} onClick={() => setPhase({ kind: 'passkey' })} data-testid="enrol-passkey">{t('settings-2fa-passkey-add')}</button>}
          </For>
        </div>
      </Show>

      <Show when={(() => { const p = phase(); return p.kind === 'totp' ? p.setup : undefined; })()}>
        {(setup) => (
          <form class={css.field} onSubmit={(e) => void confirmTotp(e)} data-testid="enrol-totp-setup">
            <p class={css.prose}>{t('login-2fa-enrol-totp-scan')}</p>
            <Show when={qr()}>{(source) => <img class={own.qr} src={source()} alt={t('login-2fa-enrol-qr-alt')} data-testid="enrol-totp-qr" />}</Show>
            <div class={css.field}>
              <span class={css.label} id="login-enrol-key">{t('login-2fa-enrol-manual-key')}</span>
              {/* As in Settings: the whole key, wrapped, never cut off. */}
              <code class={css.secret} aria-labelledby="login-enrol-key" data-testid="enrol-totp-secret">{setup().secret}</code>
            </div>
            <a class={css.meta} href={setup().otpauthUri}>{t('settings-2fa-totp-uri-link')}</a>
            <label class={css.field}>
              <span class={css.label}>{t('settings-2fa-totp-code-label')}</span>
              <input
                class={css.input}
                inputmode="numeric"
                autocomplete="one-time-code"
                value={code()}
                disabled={busy()}
                onInput={(e) => setCode(e.currentTarget.value)}
              />
            </label>
            <div class={css.actions}>
              <button type="submit" class={css.button} disabled={busy() || code().trim() === ''} data-testid="enrol-totp-confirm">
                {t('settings-2fa-totp-confirm')}
              </button>
            </div>
          </form>
        )}
      </Show>

      <Show when={phase().kind === 'passkey'}>
        <p class={css.prose}>{t('login-2fa-enrol-passkey-help')}</p>
        <label class={css.field}>
          <span class={css.label}>{t('settings-2fa-passkey-label-placeholder')}</span>
          <input class={css.input} value={label()} disabled={busy()} onInput={(e) => setLabel(e.currentTarget.value)} />
        </label>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy()} onClick={() => void enrolPasskey()} data-testid="enrol-passkey-create">
            {t('settings-2fa-passkey-add')}
          </button>
        </div>
      </Show>

      <Show when={(() => { const p = phase(); return p.kind === 'codes' ? p.codes : undefined; })()}>
        {(codes) => (
          <div class={css.codesPanel} data-testid="enrol-recovery-codes">
            <p class={css.prose} role="status">{t('login-2fa-enrol-done')}</p>
            <p class={css.meta}>{t('settings-2fa-recovery-once')}</p>
            <div class={css.codesGrid}>
              <For each={codes()}>{(c) => <span>{c}</span>}</For>
            </div>
            <label class={css.check}>
              <input type="checkbox" class={css.checkbox} checked={saved()} onChange={(e) => setSaved(e.currentTarget.checked)} data-testid="enrol-recovery-ack" />
              {t('settings-2fa-recovery-ack')}
            </label>
            <div class={css.actions}>
              <button type="button" class={css.button} disabled={!saved() || busy()} onClick={() => { setBusy(true); void Promise.resolve(props.onComplete()).finally(() => setBusy(false)); }} data-testid="enrol-continue">
                {t('login-2fa-enrol-continue')}
              </button>
            </div>
          </div>
        )}
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">{error()}</p>
      </Show>

      <Show when={canGoBack()}>
        <div class={css.actions}>
          <Show when={phase().kind !== 'choose' && methods().length > 1}>
            <button type="button" class={css.ghost} disabled={busy()} onClick={() => { setError(''); setPhase({ kind: 'choose' }); }} data-testid="enrol-other-method">
              {t('login-2fa-enrol-other-method')}
            </button>
          </Show>
          <button type="button" class="btn btn--ghost" disabled={busy()} onClick={() => props.onBack()}>
            {t('login-2fa-back')}
          </button>
        </div>
      </Show>
    </section>
  );
}

export default LoginEnrolment;
