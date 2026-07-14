// Device-pairing QR + SAS UX (SPEC §9.1, plan §3 e8). Two roles:
//   • New device  — generates an ephemeral key, shows it as a QR (and copyable text),
//                    receives the sealed envelope (relay or paste), recovers the root
//                    key, and shows the SAS words to compare.
//   • This device — scans/enters the new device's public point, seals the root key,
//                   relays (or shows) the envelope, and shows the SAS words to compare.
// The user compares the six SAS words on both screens; a match authenticates the
// channel and defeats a machine-in-the-middle relay. The server only relays ciphertext.

import { createSignal, onMount, Show, For, type JSX } from 'solid-js';
import { Qr } from './Qr.tsx';
import { PairingService, sasMatches, type PairingOffer } from './pairing.ts';
import type { ZeroAccessCrypto, ZaKeyRef } from './crypto.ts';
import type { Fetcher } from './service.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

export interface DevicePairingProps {
  za: ZeroAccessCrypto;
  /** The unlocked root key ref, when this device can act as the EXISTING device. */
  rootRef?: ZaKeyRef | undefined;
  fetcher?: Fetcher | undefined;
  onPaired?: ((rootRef: ZaKeyRef) => void) | undefined;
}

type Role = 'new' | 'existing';

export function DevicePairing(props: DevicePairingProps): JSX.Element {
  onMount(() => void loadCatalog('security'));
  const service = new PairingService(props.za, props.fetcher);
  const [role, setRole] = createSignal<Role>(props.rootRef ? 'existing' : 'new');
  const [offer, setOffer] = createSignal<PairingOffer | null>(null);
  const [envelopeIn, setEnvelopeIn] = createSignal('');
  const [peerPublic, setPeerPublic] = createSignal('');
  const [envelopeOut, setEnvelopeOut] = createSignal('');
  const [sas, setSas] = createSignal<readonly string[] | null>(null);
  const [confirmed, setConfirmed] = createSignal(false);
  const [error, setError] = createSignal('');

  async function startNewDevice(): Promise<void> {
    setError('');
    try {
      setOffer(await service.createOffer());
    } catch (e) {
      setError(e instanceof Error ? e.message : t('security-pair-err-start'));
    }
  }

  async function completeNewDevice(): Promise<void> {
    setError('');
    const o = offer();
    if (o === null) return;
    try {
      const done = await service.complete(o, envelopeIn().trim());
      setSas(done.sasWords);
      props.onPaired?.(done.rootRef);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('security-pair-err-complete'));
    }
  }

  async function sealFromExisting(): Promise<void> {
    setError('');
    if (props.rootRef === undefined) {
      setError(t('security-pair-err-unlock'));
      return;
    }
    try {
      const sealed = await service.seal(props.rootRef, peerPublic().trim());
      setEnvelopeOut(sealed.envelopeB64);
      setSas(sealed.sasWords);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('security-pair-err-seal'));
    }
  }

  return (
    <section class={css.section} aria-label={t('security-pair-title')}>
      <h3 class={css.heading}>{t('security-pair-heading')}</h3>
      <p class={css.prose}>{t('security-pair-intro')}</p>

      <div class={css.row} role="group" aria-label={t('security-pair-role')}>
        <button
          type="button"
          class={role() === 'new' ? css.button : css.buttonGhost}
          aria-pressed={role() === 'new'}
          onClick={() => setRole('new')}
        >
          {t('security-pair-new-role')}
        </button>
        <button
          type="button"
          class={role() === 'existing' ? css.button : css.buttonGhost}
          aria-pressed={role() === 'existing'}
          onClick={() => setRole('existing')}
        >
          {t('security-pair-existing-role')}
        </button>
      </div>

      <Show when={role() === 'new'}>
        <div class={css.field}>
          <Show
            when={offer()}
            fallback={
              <button type="button" class={css.button} onClick={() => void startNewDevice()}>
                {t('security-pair-show-qr')}
              </button>
            }
          >
            {(o) => (
              <>
                <span class={css.subHeading}>{t('security-pair-scan')}</span>
                <Qr value={o().publicB64} label={t('security-pair-qr-label')} />
                <label class={css.field}>
                  <span class={css.subHeading}>{t('security-pair-copy-code')}</span>
                  <input class={css.input} readOnly value={o().publicB64} data-testid="offer-public" />
                </label>
                <label class={css.field}>
                  <span class={css.subHeading}>{t('security-pair-paste-envelope')}</span>
                  <input
                    class={css.input}
                    value={envelopeIn()}
                    placeholder={t('security-pair-envelope-placeholder')}
                    onInput={(e) => setEnvelopeIn(e.currentTarget.value)}
                    aria-label={t('security-pair-envelope-label')}
                  />
                </label>
                <button
                  type="button"
                  class={css.button}
                  disabled={envelopeIn().trim() === ''}
                  onClick={() => void completeNewDevice()}
                >
                  {t('security-pair-complete')}
                </button>
              </>
            )}
          </Show>
        </div>
      </Show>

      <Show when={role() === 'existing'}>
        <div class={css.field}>
          <label class={css.field}>
            <span class={css.subHeading}>{t('security-pair-code-from-new')}</span>
            <input
              class={css.input}
              value={peerPublic()}
              placeholder={t('security-pair-code-placeholder')}
              onInput={(e) => setPeerPublic(e.currentTarget.value)}
              aria-label={t('security-pair-code-label')}
            />
          </label>
          <button
            type="button"
            class={css.button}
            disabled={peerPublic().trim() === ''}
            onClick={() => void sealFromExisting()}
          >
            {t('security-pair-seal')}
          </button>
          <Show when={envelopeOut() !== ''}>
            <label class={css.field}>
              <span class={css.subHeading}>{t('security-pair-envelope-out')}</span>
              <input class={css.input} readOnly value={envelopeOut()} data-testid="envelope-out" />
            </label>
          </Show>
        </div>
      </Show>

      <Show when={sas()}>
        {(words) => (
          <div class={css.field} data-testid="sas-block">
            <span class={css.subHeading}>{t('security-pair-compare')}</span>
            <div class={css.sasGrid} aria-label={t('security-pair-sas-label')}>
              <For each={words()}>{(w) => <span class={css.sasWord}>{w}</span>}</For>
            </div>
            <div class={css.row}>
              <button
                type="button"
                class={css.button}
                onClick={() => setConfirmed(true)}
                disabled={confirmed()}
              >
                {t('security-pair-match')}
              </button>
              <button type="button" class={css.danger} onClick={() => setSas(null)}>
                {t('security-pair-abort')}
              </button>
            </div>
            <Show when={confirmed()}>
              <p class={css.prose} data-testid="pairing-confirmed">
                {t('security-pair-confirmed')}
              </p>
            </Show>
          </div>
        )}
      </Show>

      <Show when={error() !== ''}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

/** Re-export for tests / callers verifying the SAS gate. */
export { sasMatches };
