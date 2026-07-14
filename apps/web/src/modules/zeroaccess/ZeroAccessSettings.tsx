// Zero-access settings UX (SPEC §9, plan §3 e8). Enable/disable the mode with the
// HONEST tradeoff disclosure (what the server still sees, the malicious-active-server
// caveat, the no-searchable-encryption claim, the lost-key tradeoff), passphrase OR
// passkey (WebAuthn-PRF) setup, one-time recovery-phrase display, and the device-
// pairing QR + SAS UX. All crypto runs through the injected `ZeroAccessCrypto` (the
// wasm worker in production); no plaintext key ever enters this component or the server.
//
// EXPORTED for e11 to mount into Settings/an account screen — this file does NOT touch
// the app router or Settings.tsx (ownership boundary, plan §3 e8).

import { createSignal, createResource, Show, For, type JSX } from 'solid-js';
import { ZeroAccessService, type ZeroAccessAccount, type ZeroAccessSession } from './service.ts';
import { DevicePairing } from './DevicePairing.tsx';
import { passkeySupported, passkeySecretB64 } from './passkey.ts';
import { utf8ToB64, type ZeroAccessCrypto } from './crypto.ts';
import type { Fetcher } from './service.ts';
import {
  ZA_PROTECTS,
  ZA_SERVER_STILL_SEES,
  ZA_ACTIVE_SERVER_CAVEAT,
  ZA_NO_SEARCH_CLAIM,
  ZA_RECOVERY_TRADEOFF,
} from './disclosure.ts';
import * as css from './styles.css.ts';

export interface ZeroAccessSettingsProps {
  za: ZeroAccessCrypto;
  fetcher?: Fetcher;
  /** Optional preloaded status (tests inject; production fetches). */
  initialStatus?: ZeroAccessAccount;
}

type SecretMode = 'passphrase' | 'passkey';

export function ZeroAccessSettings(props: ZeroAccessSettingsProps): JSX.Element {
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
    if (passphrase().length < 8) throw new Error('use a passphrase of at least 8 characters');
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
      setError(e instanceof Error ? e.message : 'could not enable zero-access');
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
      setError(e instanceof Error ? e.message : 'could not disable zero-access');
    } finally {
      setBusy(false);
    }
  }

  const enabled = (): boolean => status()?.enabled === true;

  return (
    <div class={css.panel} aria-label="Zero-access storage">
      <section class={css.section}>
        <div class={css.row}>
          <h2 class={css.heading}>Zero-access storage</h2>
          <Show when={enabled()} fallback={<span class={css.badgeOff}>Off</span>}>
            <span class={css.badgeOn}>On</span>
          </Show>
        </div>
        <p class={css.prose}>{ZA_PROTECTS}</p>
      </section>

      {/* HONEST disclosure — always shown, never softened (plan §1.4). */}
      <section class={css.section} aria-label="What the server still sees">
        <span class={css.subHeading}>What the server still sees</span>
        <ul class={css.list}>
          <For each={ZA_SERVER_STILL_SEES}>{(item) => <li>{item}</li>}</For>
        </ul>
        <p class={css.caveat} data-testid="active-server-caveat">
          {ZA_ACTIVE_SERVER_CAVEAT}
        </p>
        <p class={css.prose} data-testid="no-search-claim">
          {ZA_NO_SEARCH_CLAIM}
        </p>
        <p class={css.prose} data-testid="recovery-tradeoff">
          {ZA_RECOVERY_TRADEOFF}
        </p>
      </section>

      <Show when={!enabled()}>
        <section class={css.section} aria-label="Enable zero-access">
          <span class={css.subHeading}>Set up your key</span>
          <div class={css.row} role="group" aria-label="Key source">
            <button
              type="button"
              class={mode() === 'passphrase' ? css.button : css.buttonGhost}
              aria-pressed={mode() === 'passphrase'}
              onClick={() => setMode('passphrase')}
            >
              Passphrase
            </button>
            <button
              type="button"
              class={mode() === 'passkey' ? css.button : css.buttonGhost}
              aria-pressed={mode() === 'passkey'}
              onClick={() => setMode('passkey')}
              disabled={!passkeySupported()}
              title={passkeySupported() ? '' : 'passkeys are not available in this browser'}
            >
              Passkey (passwordless)
            </button>
          </div>
          <Show when={mode() === 'passphrase'}>
            <label class={css.field}>
              <span class={css.subHeading}>Passphrase</span>
              <input
                class={css.input}
                type="password"
                autocomplete="new-password"
                value={passphrase()}
                onInput={(e) => setPassphrase(e.currentTarget.value)}
                aria-label="Zero-access passphrase"
              />
            </label>
          </Show>
          <button type="button" class={css.button} disabled={busy()} onClick={() => void onEnable()}>
            Enable zero-access
          </button>
        </section>
      </Show>

      <Show when={recovery() !== ''}>
        <section class={css.section} aria-label="Recovery phrase">
          <span class={css.subHeading}>Recovery phrase — save this offline now</span>
          <p class={css.prose}>
            This is the only copy. Anyone who has it can read your data; without it (and without a
            paired device) your data cannot be recovered.
          </p>
          <div class={css.phrase} data-testid="recovery-phrase">
            {recovery()}
          </div>
        </section>
      </Show>

      <Show when={enabled()}>
        <DevicePairing za={props.za} rootRef={session()?.rootRef} fetcher={props.fetcher} />
        <section class={css.section} aria-label="Disable zero-access">
          <span class={css.subHeading}>Turn off zero-access</span>
          <p class={css.prose}>
            New data will be stored unencrypted again. Existing encrypted data stays readable only
            while you can still derive your key.
          </p>
          <button type="button" class={css.danger} disabled={busy()} onClick={() => void onDisable()}>
            Disable zero-access
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
