// Device-pairing orchestration (SPEC §9.1, plan §3 e8). A new device obtains the root
// key from an existing one through a SAS-verified, client-to-client exchange; the
// server only RELAYS an opaque sealed envelope (it never sees a plaintext key). The QR
// carries the NEW device's ephemeral public point (already public); the SAS words are
// compared out-of-band by the user to defeat a machine-in-the-middle relay.
//
// Both a QR/relay path and a manual copy/paste path (no camera/relay needed) are
// supported; the relay endpoints below are for e11 to mount.

import type { Fetcher } from './service.ts';
import type { ZeroAccessCrypto, ZaKeyRef } from './crypto.ts';

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

/** New-device state after generating its QR: waits for the sealed envelope. */
export interface PairingOffer {
  readonly pairingId: string;
  /** base64 ephemeral public point — goes in the QR / manual field. */
  readonly publicB64: string;
  /** In-worker secret ref (never leaves the worker). */
  readonly secretRef: ZaKeyRef;
}

/** Result of completing pairing on the new device (root key now in this session). */
export interface PairingCompletion {
  readonly sasWords: readonly string[];
  readonly rootRef: ZaKeyRef;
}

/** Result of sealing on the existing device: SAS to compare + the relayed envelope. */
export interface PairingSeal {
  readonly sasWords: readonly string[];
  readonly envelopeB64: string;
}

/**
 * Drives both roles of the pairing ceremony over the injected worker + relay.
 */
export class PairingService {
  constructor(
    private readonly za: ZeroAccessCrypto,
    private readonly fetcher: Fetcher = defaultFetcher,
  ) {}

  /** NEW DEVICE step 1: generate the ephemeral key + register the offer for relay. */
  async createOffer(): Promise<PairingOffer> {
    const { publicB64, secretRef } = await this.za.pairGenerate();
    const res = await this.fetcher('/api/zeroaccess/pair/offer', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ publicB64 }),
    });
    const { pairingId } = (await res.json()) as { pairingId: string };
    return { pairingId, publicB64, secretRef };
  }

  /** EXISTING DEVICE: seal the root key to the scanned/entered public point. */
  async seal(rootRef: ZaKeyRef, peerPublicB64: string): Promise<PairingSeal> {
    const { sasWords, envelopeB64 } = await this.za.pairSeal({ rootRef, peerPublicB64 });
    return { sasWords, envelopeB64 };
  }

  /** EXISTING DEVICE: relay the sealed envelope back to the new device. */
  async relayEnvelope(pairingId: string, envelopeB64: string): Promise<void> {
    await this.fetcher('/api/zeroaccess/pair/envelope', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ pairingId, envelopeB64 }),
    });
  }

  /** NEW DEVICE: fetch the relayed envelope (poll until present). */
  async fetchEnvelope(pairingId: string): Promise<string | null> {
    const res = await this.fetcher(`/api/zeroaccess/pair/envelope/${encodeURIComponent(pairingId)}`);
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`pairing relay failed: ${res.status}`);
    const { envelopeB64 } = (await res.json()) as { envelopeB64: string | null };
    return envelopeB64;
  }

  /** NEW DEVICE step 2: open the envelope → recover the root key + SAS to compare. */
  async complete(offer: PairingOffer, envelopeB64: string): Promise<PairingCompletion> {
    const { sasWords, keyRef } = await this.za.pairComplete({ envelopeB64, secretRef: offer.secretRef });
    return { sasWords, rootRef: keyRef };
  }
}

/** True iff both SAS word lists match exactly (the user's confirmation gate). */
export function sasMatches(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((w, i) => w === b[i]);
}
