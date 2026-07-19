// Login-time second-factor challenge (t16 e15, SPEC §7.4/§19 — S1 web half).
//
// Self-contained + presentation-only over the injected service so it can be
// mounted by the login screen: when `/api/login` answers `twofaRequired`, the
// login owner renders this with the returned `LoginChallenge` and, on success,
// re-runs its session bootstrap. Kept OUT of the login screen's own file to
// respect the ownership boundary — export from `index.ts`, mount there. No
// downgrade: the user MUST clear a factor here; there is no password-only path.

import { createSignal, Show, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { SettingsService } from './service.ts';
import { passkeySupported, assertPasskey } from './webauthn.ts';
import type { LoginChallenge } from './types.ts';
import * as css from './styles.css.ts';

export interface TwoFactorChallengeProps {
  challenge: LoginChallenge;
  /** Called once a factor verifies and the session is issued. */
  onSuccess: () => void;
  service?: SettingsService;
}

export function TwoFactorChallenge(props: TwoFactorChallengeProps): JSX.Element {
  const service = props.service ?? new SettingsService();
  onMount(() => void loadCatalog('settings'));

  const has = (f: string): boolean => props.challenge.factors.includes(f);
  const [mode, setMode] = createSignal<'totp' | 'recovery'>(has('totp') ? 'totp' : 'recovery');
  const [code, setCode] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function fail(e: unknown): void {
    // Uniform failure copy — never reveal which check failed (mirrors the server).
    setError(e instanceof Error && e.message !== '' ? e.message : t('settings-2fa-challenge-failed'));
  }

  async function submitCode(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.verifyLoginFactor({
        pendingToken: props.challenge.pendingToken,
        method: mode(),
        code: code().trim(),
      });
      props.onSuccess();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  async function submitPasskey(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const wa = props.challenge.webauthn;
      if (wa === undefined) throw new Error(t('settings-2fa-challenge-failed'));
      const assertion = await assertPasskey(wa);
      await service.verifyLoginFactor({
        pendingToken: props.challenge.pendingToken,
        method: 'webauthn',
        credentialId: assertion.credentialId,
        clientDataJson: assertion.clientDataJson,
        authenticatorData: assertion.authenticatorData,
        signature: assertion.signature,
      });
      props.onSuccess();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  }

  const codeLabel = (): string =>
    mode() === 'totp' ? t('settings-2fa-totp-code-label') : t('settings-2fa-recovery-code-label');

  return (
    <section class={css.section} aria-label={t('settings-2fa-challenge-title')} data-testid="twofa-challenge">
      <h2 class={css.heading}>{t('settings-2fa-challenge-title')}</h2>
      <p class={css.prose}>{t('settings-2fa-challenge-intro')}</p>

      <Show when={has('webauthn') && passkeySupported() && props.challenge.webauthn}>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy()} onClick={() => void submitPasskey()} data-testid="challenge-passkey">
            {t('settings-2fa-challenge-use-passkey')}
          </button>
        </div>
      </Show>

      <Show when={has('totp') || has('recovery')}>
        <div class={css.options} role="group" aria-label={t('settings-2fa-challenge-method')}>
          <Show when={has('totp')}>
            <button type="button" class={css.option} aria-pressed={mode() === 'totp'} onClick={() => setMode('totp')}>
              {t('settings-2fa-challenge-totp-tab')}
            </button>
          </Show>
          <Show when={has('recovery')}>
            <button type="button" class={css.option} aria-pressed={mode() === 'recovery'} onClick={() => setMode('recovery')}>
              {t('settings-2fa-challenge-recovery-tab')}
            </button>
          </Show>
        </div>

        <label class={css.field}>
          <span class={css.label}>{codeLabel()}</span>
          <input
            class={css.input}
            inputmode={mode() === 'totp' ? 'numeric' : 'text'}
            autocomplete="one-time-code"
            aria-label={codeLabel()}
            value={code()}
            onInput={(e) => setCode(e.currentTarget.value)}
          />
        </label>
        <div class={css.actions}>
          <button type="button" class={css.button} disabled={busy() || code().trim() === ''} onClick={() => void submitCode()} data-testid="challenge-verify">
            {t('settings-2fa-challenge-verify')}
          </button>
        </div>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

export default TwoFactorChallenge;
