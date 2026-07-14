// Device-pairing QR + SAS UX (SPEC §9.1, plan §3 e8). Two roles:
//   • New device  — generates an ephemeral key, shows it as a QR (and copyable text),
//                    receives the sealed envelope (relay or paste), recovers the root
//                    key, and shows the SAS words to compare.
//   • This device — scans/enters the new device's public point, seals the root key,
//                   relays (or shows) the envelope, and shows the SAS words to compare.
// The user compares the six SAS words on both screens; a match authenticates the
// channel and defeats a machine-in-the-middle relay. The server only relays ciphertext.

import { createSignal, Show, For, type JSX } from 'solid-js';
import { Qr } from './Qr.tsx';
import { PairingService, sasMatches, type PairingOffer } from './pairing.ts';
import type { ZeroAccessCrypto, ZaKeyRef } from './crypto.ts';
import type { Fetcher } from './service.ts';
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
      setError(e instanceof Error ? e.message : 'could not start pairing');
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
      setError(e instanceof Error ? e.message : 'could not complete pairing');
    }
  }

  async function sealFromExisting(): Promise<void> {
    setError('');
    if (props.rootRef === undefined) {
      setError('unlock zero-access first to pair another device');
      return;
    }
    try {
      const sealed = await service.seal(props.rootRef, peerPublic().trim());
      setEnvelopeOut(sealed.envelopeB64);
      setSas(sealed.sasWords);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'could not seal the root key');
    }
  }

  return (
    <section class={css.section} aria-label="Device pairing">
      <h3 class={css.heading}>Pair a device</h3>
      <p class={css.prose}>
        Move your keys to another device without sending them through the server — it relays only an
        opaque sealed envelope. Compare the six words on both screens before trusting the pairing.
      </p>

      <div class={css.row} role="group" aria-label="Pairing role">
        <button
          type="button"
          class={role() === 'new' ? css.button : css.buttonGhost}
          aria-pressed={role() === 'new'}
          onClick={() => setRole('new')}
        >
          This is the new device
        </button>
        <button
          type="button"
          class={role() === 'existing' ? css.button : css.buttonGhost}
          aria-pressed={role() === 'existing'}
          onClick={() => setRole('existing')}
        >
          Pair another device from here
        </button>
      </div>

      <Show when={role() === 'new'}>
        <div class={css.field}>
          <Show
            when={offer()}
            fallback={
              <button type="button" class={css.button} onClick={() => void startNewDevice()}>
                Show pairing QR
              </button>
            }
          >
            {(o) => (
              <>
                <span class={css.subHeading}>Scan this on your existing device</span>
                <Qr value={o().publicB64} label="Device pairing QR code" />
                <label class={css.field}>
                  <span class={css.subHeading}>Or copy this pairing code</span>
                  <input class={css.input} readOnly value={o().publicB64} data-testid="offer-public" />
                </label>
                <label class={css.field}>
                  <span class={css.subHeading}>Paste the sealed envelope from the other device</span>
                  <input
                    class={css.input}
                    value={envelopeIn()}
                    placeholder="envelope…"
                    onInput={(e) => setEnvelopeIn(e.currentTarget.value)}
                    aria-label="Sealed envelope"
                  />
                </label>
                <button
                  type="button"
                  class={css.button}
                  disabled={envelopeIn().trim() === ''}
                  onClick={() => void completeNewDevice()}
                >
                  Complete pairing
                </button>
              </>
            )}
          </Show>
        </div>
      </Show>

      <Show when={role() === 'existing'}>
        <div class={css.field}>
          <label class={css.field}>
            <span class={css.subHeading}>Pairing code from the new device</span>
            <input
              class={css.input}
              value={peerPublic()}
              placeholder="pairing code…"
              onInput={(e) => setPeerPublic(e.currentTarget.value)}
              aria-label="Pairing code"
            />
          </label>
          <button
            type="button"
            class={css.button}
            disabled={peerPublic().trim() === ''}
            onClick={() => void sealFromExisting()}
          >
            Seal my keys for that device
          </button>
          <Show when={envelopeOut() !== ''}>
            <label class={css.field}>
              <span class={css.subHeading}>Sealed envelope — paste this on the new device</span>
              <input class={css.input} readOnly value={envelopeOut()} data-testid="envelope-out" />
            </label>
          </Show>
        </div>
      </Show>

      <Show when={sas()}>
        {(words) => (
          <div class={css.field} data-testid="sas-block">
            <span class={css.subHeading}>Compare these words on both devices</span>
            <div class={css.sasGrid} aria-label="Short authentication string">
              <For each={words()}>{(w) => <span class={css.sasWord}>{w}</span>}</For>
            </div>
            <div class={css.row}>
              <button
                type="button"
                class={css.button}
                onClick={() => setConfirmed(true)}
                disabled={confirmed()}
              >
                The words match
              </button>
              <button type="button" class={css.danger} onClick={() => setSas(null)}>
                They differ — abort
              </button>
            </div>
            <Show when={confirmed()}>
              <p class={css.prose} data-testid="pairing-confirmed">
                Pairing confirmed. This channel is authenticated.
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
