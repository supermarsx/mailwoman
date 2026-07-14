// Zero-access settings UX (SPEC §9, plan §3 e8). Enable/disable the mode with the
// HONEST tradeoff disclosure (what the server still sees, the malicious-active-server
// caveat, the no-searchable-encryption claim, the lost-key tradeoff), passphrase OR
// passkey (WebAuthn-PRF) setup, one-time recovery-phrase display, and the device-
// pairing QR + SAS UX. All crypto runs through the injected `ZeroAccessCrypto` (the
// wasm worker in production); no plaintext key ever enters this component or the server.
//
// EXPORTED for e11 to mount into Settings/an account screen — this file does NOT touch
// the app router or Settings.tsx (ownership boundary, plan §3 e8).

import { createSignal, createResource, onMount, Show, For, type JSX } from 'solid-js';
import { ZeroAccessService, type ZeroAccessAccount, type ZeroAccessSession } from './service.ts';
import { DevicePairing } from './DevicePairing.tsx';
import { passkeySupported, passkeySecretB64 } from './passkey.ts';
import { utf8ToB64, type ZeroAccessCrypto } from './crypto.ts';
import type { Fetcher } from './service.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

// The honest zero-access disclosure copy is authored in `locales/en/security.ftl`
// (security-za-*) and rendered via `t()`. Its English is byte-identical to the
// canonical constants in `./disclosure.ts` — the honesty unit tests compare the two.
// A list of the five "what the server still sees" ids, in display order.
const ZA_SEES_IDS = [
  'security-za-sees-1',
  'security-za-sees-2',
  'security-za-sees-3',
  'security-za-sees-4',
  'security-za-sees-5',
] as const;

export interface ZeroAccessSettingsProps {
  za: ZeroAccessCrypto;
  fetcher?: Fetcher;
  /** Optional preloaded status (tests inject; production fetches). */
  initialStatus?: ZeroAccessAccount;
}

type SecretMode = 'passphrase' | 'passkey';

export function ZeroAccessSettings(props: ZeroAccessSettingsProps): JSX.Element {
  onMount(() => void loadCatalog('security'));
  const service = new ZeroAccessService(props.za, props.fetcher);
  const [status, { refetch }] = createResource<ZeroAccessAccount>(
    () => props.initialStatus ?? service.status(),
  );

  const [mode, setMode] = createSignal<SecretMode>('passphrase');
  const [passphrase, setPassphrase] = createSignal('');
  const [session, setSession] = createSignal<ZeroAccessSession | null>(null);
  const [recovery, setRecovery] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  async function secretB64(): Promise<string> {
    if (mode() === 'passkey') {
      // The registered credential id + challenge come from the server in production;
      // here we surface a clear error if the environment lacks WebAuthn/PRF.
      const challenge = new Uint8Array(32);
      crypto.getRandomValues(challenge);
      return passkeySecretB64('', challenge);
    }
    if (passphrase().length < 8) throw new Error(t('security-za-err-passphrase-len'));
    return utf8ToB64(passphrase());
  }

  async function onEnable(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const sess = await service.enable(await secretB64());
      setSession(sess);
      setRecovery(await service.recoveryPhrase(sess));
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('security-za-err-enable'));
    } finally {
      setBusy(false);
    }
  }

  async function onDisable(): Promise<void> {
    setError('');
    setBusy(true);
    try {
      await service.disable();
      setSession(null);
      setRecovery('');
      await refetch();
    } catch (e) {
      setError(e instanceof Error ? e.message : t('security-za-err-disable'));
    } finally {
      setBusy(false);
    }
  }

  const enabled = (): boolean => status()?.enabled === true;

  return (
    <div class={css.panel} aria-label={t('security-za-title')}>
      <section class={css.section}>
        <div class={css.row}>
          <h2 class={css.heading}>{t('security-za-title')}</h2>
          <Show when={enabled()} fallback={<span class={css.badgeOff}>{t('security-za-off')}</span>}>
            <span class={css.badgeOn}>{t('security-za-on')}</span>
          </Show>
        </div>
        <p class={css.prose}>{t('security-za-protects')}</p>
      </section>

      {/* HONEST disclosure — always shown, never softened (plan §1.4). */}
      <section class={css.section} aria-label={t('security-za-server-sees-title')}>
        <span class={css.subHeading}>{t('security-za-server-sees-title')}</span>
        <ul class={css.list}>
          <For each={ZA_SEES_IDS}>{(id) => <li>{t(id)}</li>}</For>
        </ul>
        <p class={css.caveat} data-testid="active-server-caveat">
          {t('security-za-active-server-caveat')}
        </p>
        <p class={css.prose} data-testid="no-search-claim">
          {t('security-za-no-search-claim')}
        </p>
        <p class={css.prose} data-testid="recovery-tradeoff">
          {t('security-za-recovery-tradeoff')}
        </p>
      </section>

      <Show when={!enabled()}>
        <section class={css.section} aria-label={t('security-za-enable')}>
          <span class={css.subHeading}>{t('security-za-setup-key')}</span>
          <div class={css.row} role="group" aria-label={t('security-za-key-source')}>
            <button
              type="button"
              class={mode() === 'passphrase' ? css.button : css.buttonGhost}
              aria-pressed={mode() === 'passphrase'}
              onClick={() => setMode('passphrase')}
            >
              {t('security-za-passphrase')}
            </button>
            <button
              type="button"
              class={mode() === 'passkey' ? css.button : css.buttonGhost}
              aria-pressed={mode() === 'passkey'}
              onClick={() => setMode('passkey')}
              disabled={!passkeySupported()}
              title={passkeySupported() ? '' : t('security-za-passkey-unavailable')}
            >
              {t('security-za-passkey')}
            </button>
          </div>
          <Show when={mode() === 'passphrase'}>
            <label class={css.field}>
              <span class={css.subHeading}>{t('security-za-passphrase')}</span>
              <input
                class={css.input}
                type="password"
                autocomplete="new-password"
                value={passphrase()}
                onInput={(e) => setPassphrase(e.currentTarget.value)}
                aria-label={t('security-za-passphrase-label')}
              />
            </label>
          </Show>
          <button type="button" class={css.button} disabled={busy()} onClick={() => void onEnable()}>
            {t('security-za-enable')}
          </button>
        </section>
      </Show>

      <Show when={recovery() !== ''}>
        <section class={css.section} aria-label={t('security-za-recovery-title')}>
          <span class={css.subHeading}>{t('security-za-recovery-heading')}</span>
          <p class={css.prose}>{t('security-za-recovery-note')}</p>
          <div class={css.phrase} data-testid="recovery-phrase">
            {recovery()}
          </div>
        </section>
      </Show>

      <Show when={enabled()}>
        <DevicePairing za={props.za} rootRef={session()?.rootRef} fetcher={props.fetcher} />
        <section class={css.section} aria-label={t('security-za-disable-section')}>
          <span class={css.subHeading}>{t('security-za-disable-heading')}</span>
          <p class={css.prose}>{t('security-za-disable-note')}</p>
          <button type="button" class={css.danger} disabled={busy()} onClick={() => void onDisable()}>
            {t('security-za-disable-btn')}
          </button>
        </section>
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </div>
  );
}

export default ZeroAccessSettings;
