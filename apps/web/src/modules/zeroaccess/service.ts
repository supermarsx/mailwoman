// Zero-access orchestration + server I/O (plan §3 e8). Composes the `za*` worker calls
// into the SPEC §9.1 flows (enable/unlock/seal/open/pairing) and talks to the server
// endpoints e11 will mount. The server stores only wrapped keys + ciphertext + opaque
// metadata; no plaintext key is ever sent (plan §1.2 / §1.4).
//
// Every network call is injectable (`fetcher`) so components stay unit-testable without
// a live server; the default uses same-origin cookie auth like `api/client.ts`.

import {
  ZA_KDF_INTERACTIVE,
  ZA_SUBKEY_LABELS,
  utf8ToB64,
  b64ToUtf8,
  type ZaKdfParams,
  type ZaKeyRef,
  type ZaSubkeyLabel,
  type ZeroAccessCrypto,
} from './crypto.ts';

/** A device paired to a zero-access account (opaque to the server; §9.1). */
export interface PairedDevice {
  readonly id: string;
  readonly label: string;
  readonly pairedAt: string;
}

/** The persisted zero-access state for the current account (`zeroaccess_accounts`). */
export interface ZeroAccessAccount {
  readonly enabled: boolean;
  readonly saltB64?: string;
  readonly kdfParams?: ZaKdfParams;
  /** KEK-wrapped per-account data key (base64). */
  readonly wrappedDataKeyB64?: string;
  readonly pairedDevices: readonly PairedDevice[];
}

/** The payload POSTed to enable zero-access — wrapped material only, never a raw key. */
export interface ZeroAccessEnablePayload {
  readonly saltB64: string;
  readonly kdfParams: ZaKdfParams;
  readonly wrappedDataKeyB64: string;
}

/** Injected transport (default = same-origin cookie fetch). */
export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`zero-access request failed: ${res.status}`);
  return (await res.json()) as T;
}

function randomSaltB64(): string {
  const salt = new Uint8Array(16);
  crypto.getRandomValues(salt);
  let bin = '';
  for (const b of salt) bin += String.fromCharCode(b);
  return btoa(bin);
}

/** An unlocked zero-access session: the in-worker key refs for one account. */
export interface ZeroAccessSession {
  readonly rootRef: ZaKeyRef;
  readonly kekRef: ZaKeyRef;
  readonly dataKeyRef: ZaKeyRef;
  /** Per-class subkeys derived from the data key (message-cache/search/notes/attachment). */
  readonly subkeys: Readonly<Record<ZaSubkeyLabel, ZaKeyRef>>;
}

/**
 * The service backing the zero-access UI. All crypto goes through the injected
 * [`ZeroAccessCrypto`] (the wasm worker in production, a mock in tests); all network
 * I/O goes through the injected [`Fetcher`].
 */
export class ZeroAccessService {
  constructor(
    private readonly za: ZeroAccessCrypto,
    private readonly fetcher: Fetcher = defaultFetcher,
  ) {}

  /** Current server-side zero-access state for this account. */
  async status(): Promise<ZeroAccessAccount> {
    const res = await this.fetcher('/api/zeroaccess');
    return jsonOrThrow<ZeroAccessAccount>(res);
  }

  private async deriveSubkeys(dataKeyRef: ZaKeyRef): Promise<Record<ZaSubkeyLabel, ZaKeyRef>> {
    const entries = await Promise.all(
      ZA_SUBKEY_LABELS.map(async (label) => {
        const { keyRef } = await this.za.deriveSubkey({ keyRef: dataKeyRef, label });
        return [label, keyRef] as const;
      }),
    );
    return Object.fromEntries(entries) as Record<ZaSubkeyLabel, ZaKeyRef>;
  }

  /**
   * ENABLE zero-access for this account: derive the root from the passphrase (or PRF
   * secret), a KEK, and a fresh per-account data key; wrap the data key under the KEK;
   * and POST only the wrapped material + KDF params. Returns the unlocked session.
   */
  async enable(secretB64: string, kdf: ZaKdfParams = ZA_KDF_INTERACTIVE): Promise<ZeroAccessSession> {
    const saltB64 = randomSaltB64();
    const { keyRef: rootRef } = await this.za.deriveRootKey({
      secretB64,
      saltB64,
      mCost: kdf.mCost,
      tCost: kdf.tCost,
      pCost: kdf.pCost,
    });
    const { keyRef: kekRef } = await this.za.deriveKek({ keyRef: rootRef });
    const { keyRef: dataKeyRef } = await this.za.generateDataKey();
    const { blobB64: wrappedDataKeyB64 } = await this.za.wrapKey({ kekRef, dataKeyRef });
    const payload: ZeroAccessEnablePayload = { saltB64, kdfParams: kdf, wrappedDataKeyB64 };
    await this.fetcher('/api/zeroaccess/enable', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const subkeys = await this.deriveSubkeys(dataKeyRef);
    return { rootRef, kekRef, dataKeyRef, subkeys };
  }

  /**
   * UNLOCK an already-enabled account on login: re-derive the root from the passphrase +
   * stored salt/KDF, re-derive the KEK, and unwrap the stored per-account data key.
   */
  async unlock(secretB64: string, account: ZeroAccessAccount): Promise<ZeroAccessSession> {
    if (!account.enabled || account.saltB64 === undefined || account.kdfParams === undefined || account.wrappedDataKeyB64 === undefined) {
      throw new Error('account is not zero-access enabled');
    }
    const { keyRef: rootRef } = await this.za.deriveRootKey({
      secretB64,
      saltB64: account.saltB64,
      mCost: account.kdfParams.mCost,
      tCost: account.kdfParams.tCost,
      pCost: account.kdfParams.pCost,
    });
    const { keyRef: kekRef } = await this.za.deriveKek({ keyRef: rootRef });
    const { keyRef: dataKeyRef } = await this.za.unwrapKey({ kekRef, blobB64: account.wrappedDataKeyB64 });
    const subkeys = await this.deriveSubkeys(dataKeyRef);
    return { rootRef, kekRef, dataKeyRef, subkeys };
  }

  /** Seal one plaintext row before it is written to the (zero-access) store. */
  async sealRow(session: ZeroAccessSession, label: ZaSubkeyLabel, plaintext: string, table: string, rowId: string, schemaVersion: number): Promise<string> {
    const { ciphertextB64 } = await this.za.sealRow({
      keyRef: session.subkeys[label],
      plaintextB64: utf8ToB64(plaintext),
      table,
      rowId,
      schemaVersion,
    });
    return ciphertextB64;
  }

  /** Open one ciphertext row read back from the store (AAD must match its location). */
  async openRow(session: ZeroAccessSession, label: ZaSubkeyLabel, ciphertextB64: string, table: string, rowId: string, schemaVersion: number): Promise<string> {
    const { plaintextB64 } = await this.za.openRow({
      keyRef: session.subkeys[label],
      ciphertextB64,
      table,
      rowId,
      schemaVersion,
    });
    return b64ToUtf8(plaintextB64);
  }

  /** The user-initiated recovery phrase (offline backup) for the root key. */
  async recoveryPhrase(session: ZeroAccessSession): Promise<string> {
    const { phrase } = await this.za.recoveryPhrase({ keyRef: session.rootRef });
    return phrase;
  }

  /** DISABLE zero-access for this account (server drops the wrapped material). */
  async disable(): Promise<void> {
    await this.fetcher('/api/zeroaccess/disable', { method: 'POST' });
    await this.za.lockAll();
  }

  /** Clear the in-worker key session (logout / idle timeout). */
  async lock(): Promise<void> {
    await this.za.lockAll();
  }
}

/** Convert a `ZaKdfParams` back into a human summary for the disclosure UI. */
export function describeKdf(kdf: ZaKdfParams): string {
  return `Argon2id · ${Math.round(kdf.mCost / 1024)} MiB · ${kdf.tCost} pass · ${kdf.pCost} lane`;
}
