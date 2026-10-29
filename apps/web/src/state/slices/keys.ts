// Keys store slice (plan §2.5, §3 e0 stub → e2 fills). Owns the `CryptoKey/*`
// surface for the web client: the own + contact/harvested key list, generate
// (OpenPGP, or an S/MIME key with a self-signed certificate), import (armored +
// PKCS#12), backup (Autocrypt Setup Message; PKCS#12 export, certification
// request and issued-certificate replacement for S/MIME), trust/verify,
// WKD/VKS/harvest lookup, and per-contact key association. Disjoint file — no
// `store.ts` collision with the other slices (same discipline as the V2/V3 slices).
//
// PRIVACY (plan §1.2 / risk #4): private-key generation/import runs in the crypto
// WORKER (`crypto/index.ts`), and the wrapped private bundle lives ONLY in the
// client `KeyVault`; this slice + the server hold PUBLIC key metadata plus the
// OPAQUE `encryptedPrivateBackup` (which the server never decrypts). e2 fills the
// full flows against the mock + the worker STUB; e8 swaps to the real engine +
// wasm worker (by flipping `getCryptoWorker()` — this slice is unchanged then).

import { createSignal, getOwner, onCleanup, type Accessor } from 'solid-js';
import { observePreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { onLinkedMailboxChange } from '../../api/linkedMailboxSession.ts';
import { CAP_CORE, type Id, type JmapRequest } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_CONTACTS } from '../../api/pim-types.ts';
import {
  CAP_CRYPTO,
  CAP_SECURITY,
  type CryptoKey,
  type KeyKind,
  type KeySource,
  type KeyTrust,
} from '../../api/crypto-types.ts';
import { getCryptoWorker, createInMemoryVault, type KeyVault } from '../../crypto/index.ts';
import type { KeyRef } from '../../contracts/crypto.ts';
import type { SliceContext } from './context.ts';

const KEYS_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

/** A source the keyring can look a contact key up from (§2.2 `CryptoKey/lookup`). */
export type KeyLookupSource = 'wkd' | 'vks' | 'autocrypt' | 'harvested';

/**
 * The fields a new own-key generation collects from the UI. `'pgp'` makes an
 * OpenPGP key. `'smime'` makes an RSA key and an X.509 certificate signed by that
 * key itself — a real certificate, but one no certificate authority vouches for
 * (`smimeCertificateRequest` produces the file an authority issues from, and
 * `attachIssuedCertificate` stores what it sends back).
 */
export interface OwnKeyDraft {
  kind: KeyKind;
  /** A user id (`Name <email>` or a bare address). */
  userId: string;
  passphrase: string;
}

/** A parsed-but-not-yet-persisted imported key (the import "preview" step). */
export interface ImportPreview {
  key: CryptoKey;
  /** The opaque wrapped private bundle, present iff the import carried a key. */
  encryptedPrivateBundle: string | null;
}

/** The CryptoKey props the key list needs (all frozen §2.1 fields). */
const KEY_PROPERTIES = [
  'id',
  'kind',
  'isOwn',
  'addresses',
  'fingerprint',
  'keyId',
  'algorithm',
  'createdAt',
  'expiresAt',
  'publicKeyArmored',
  'certPem',
  'trust',
  'autocrypt',
  'source',
  'hasPrivate',
  'encryptedPrivateBackup',
  'verifiedAt',
  'keyHistory',
] as const;

/** Query the account's keys, then hydrate in one round-trip (CryptoKey/query→get). */
export function keyQuery(accountId: Id): JmapRequest {
  return {
    using: KEYS_USING,
    methodCalls: [
      ['CryptoKey/query', { accountId }, 'q'],
      [
        'CryptoKey/get',
        { accountId, '#ids': { resultOf: 'q', name: 'CryptoKey/query', path: '/ids' }, properties: [...KEY_PROPERTIES] },
        'g',
      ],
    ],
  };
}

interface KeyGetResponse {
  accountId: Id;
  state: string;
  list: CryptoKey[];
  notFound: Id[];
}

interface KeySetResponse {
  accountId: Id;
  created: Record<string, { id: Id } & Partial<CryptoKey>> | null;
}

interface KeyLookupResponse {
  accountId: Id;
  list: CryptoKey[];
  notFound: string[];
}

/** Extract the bare email from a `Name <email>` user id (or return it as-is). */
export function addressOf(userId: string): string {
  const m = userId.match(/<([^>]+)>/);
  return (m?.[1] ?? userId).trim();
}

/** The public interface of the keys slice (e0 stub; e2 extends). */
export interface KeysSlice {
  keys: Accessor<CryptoKey[]>;
  keysLoading: Accessor<boolean>;
  /** Own keys (generated/imported); the subset with `isOwn`. */
  ownKeys: Accessor<CryptoKey[]>;
  /** Contact/harvested/looked-up keys (the non-own subset). */
  contactKeys: Accessor<CryptoKey[]>;
  /** Load the account's keys (own + contact). */
  loadKeys(): Promise<void>;
  /** Set a key's TOFU trust (verify/revoke). */
  setKeyTrust(id: Id, trust: KeyTrust): Promise<void>;
  /** Generate a new own key in the crypto worker; wraps + persists it. */
  generateOwnKey(draft: OwnKeyDraft): Promise<CryptoKey>;
  /** Parse an armored key (worker), returning a preview WITHOUT persisting. */
  previewArmoredKey(armored: string, passphrase?: string): Promise<ImportPreview>;
  /** Parse a PKCS#12 blob (worker), returning a preview WITHOUT persisting. */
  previewPkcs12Key(p12Bytes: Uint8Array, password: string): Promise<ImportPreview>;
  /** Commit a previewed import: vault-store any bundle + persist the public key. */
  commitImport(preview: ImportPreview): Promise<CryptoKey>;
  /** Export an own key as an Autocrypt Setup Message (needs the vaulted bundle). */
  exportKeyBackup(fingerprint: string): Promise<string>;
  /**
   * An own S/MIME key and its certificate as PKCS#12 (`.p12`) bytes, protected by
   * the key's passphrase — the form other mail programs import.
   */
  exportSmimePkcs12(id: Id, passphrase: string): Promise<Uint8Array>;
  /** A PKCS#10 certification request (PEM) for an own S/MIME key. */
  smimeCertificateRequest(id: Id, passphrase: string): Promise<string>;
  /**
   * Replace an own S/MIME key's certificate with one issued for the same key
   * (PEM or DER). Rejects, storing nothing, when the certificate is for another key.
   */
  attachIssuedCertificate(id: Id, certBytes: Uint8Array, passphrase: string): Promise<CryptoKey>;
  /** Look a contact key up over WKD/VKS/autocrypt/harvest (consent-gated in UI). */
  lookupContactKey(address: string, sources: KeyLookupSource[]): Promise<CryptoKey[]>;
  /** Write a key onto a V3 contact card (`pgpKey`/`smimeCert`), populating it. */
  associateKeyWithContact(contactId: Id, key: CryptoKey): Promise<void>;
  /** Whether a wrapped private bundle for `fingerprint` is held in the vault. */
  hasVaultedKey(fingerprint: string): boolean;
  /**
   * The account whose keys this slice currently holds (t87 D11), or `null`
   * before the first load and right after a mailbox switch.
   */
  keysAccountId: Accessor<string | null>;
  /**
   * The account a loaded key belongs to, or `null` for a key object this slice
   * did not load for the current mailbox (for instance one kept from before a
   * switch). Compose and Reader refuse to use a key whose owner is not the
   * mailbox they act for.
   */
  keyOwnerAccount(key: CryptoKey): string | null;
  /**
   * Unlock an own key of the CURRENT mailbox in the crypto worker. The ref is
   * recorded against that account and locked when the mailbox changes.
   * Rejects when the key is not one this slice loaded for the current mailbox.
   */
  unlockOwnKey(key: CryptoKey, passphrase: string): Promise<{ keyRef: KeyRef; accountId: string }>;
  /** Lock one worker key ref issued by `unlockOwnKey`. */
  lockKeyRef(keyRef: KeyRef): Promise<void>;
  /**
   * Increases on every mailbox switch. Holders of an unlocked key or a cached
   * passphrase drop them when it changes.
   */
  keysEpoch: Accessor<number>;
}

/**
 * The private-key vaults, one per mailbox (t87 D11), keyed by account id. A
 * switch drops them all: the next mailbox starts locked and empty and unlocks
 * with its own passphrase.
 */
export interface MailboxVaults {
  vaultFor(accountId: string): KeyVault;
  /** Remove every entry of every vault and forget the vaults. */
  dropAll(): Promise<void>;
  /** The account ids that currently have a vault. */
  accounts(): string[];
}

export function createMailboxVaults(): MailboxVaults {
  const vaults = new Map<string, KeyVault>();
  return {
    vaultFor(accountId: string): KeyVault {
      let v = vaults.get(accountId);
      if (v === undefined) {
        v = createInMemoryVault();
        vaults.set(accountId, v);
      }
      return v;
    },
    async dropAll(): Promise<void> {
      const all = [...vaults.values()];
      vaults.clear();
      for (const v of all) for (const e of await v.list()) await v.remove(e.fingerprint);
    },
    accounts: () => [...vaults.keys()],
  };
}

/** Thrown when a key operation started for one mailbox finishes after a switch. */
function mailboxChanged(): DOMException {
  return new DOMException('Mailbox changed', 'AbortError');
}

export function createKeysSlice(ctx: SliceContext): KeysSlice {
  const client = ctx.client;
  const worker = getCryptoWorker();
  // One vault per mailbox (t87 D11), keyed by account id; see `vault()`.
  const vaults = createMailboxVaults();
  const [keys, setKeys] = createSignal<CryptoKey[]>([]);
  const [keysLoading, setKeysLoading] = createSignal(false);
  const [accountId, setAccountId] = createSignal<string | null>(null);
  const [vaulted, setVaulted] = createSignal<Set<string>>(new Set());
  const [keysEpoch, setKeysEpoch] = createSignal(0);
  /** Which account each loaded key object belongs to. */
  let owners = new WeakMap<CryptoKey, string>();
  /** Worker key refs unlocked through this slice, with their account. */
  const issued = new Map<KeyRef, string>();
  let epoch = 0;

  const ownKeys = (): CryptoKey[] => keys().filter((k) => k.isOwn);
  const contactKeys = (): CryptoKey[] => keys().filter((k) => !k.isOwn);
  const hasVaultedKey = (fingerprint: string): boolean => vaulted().has(fingerprint);

  /** The vault of `acct`, which must be the current mailbox's account. */
  function vault(acct: string): KeyVault {
    if (acct !== accountId()) throw mailboxChanged();
    return vaults.vaultFor(acct);
  }

  /** Record `list` as keys of `acct`, tagging each object with its owner. */
  function tagged(acct: string, list: CryptoKey[] | undefined): CryptoKey[] {
    const rows = Array.isArray(list) ? list : [];
    for (const k of rows) owners.set(k, acct);
    return rows;
  }

  /**
   * A mailbox switch (t87 D11): lock every key the worker holds for the old
   * mailbox, drop its vault, the loaded keys and the cached account, and make
   * any operation still in flight for it finish without effect.
   */
  function resetForSwitch(): void {
    epoch += 1;
    const refs = [...issued.keys()];
    issued.clear();
    for (const keyRef of refs) void worker.lockKey({ keyRef }).catch(() => undefined);
    void vaults.dropAll();
    owners = new WeakMap();
    setKeys([]);
    setVaulted(new Set<string>());
    setAccountId(null);
    setKeysLoading(false);
    setKeysEpoch((n) => n + 1);
  }

  // The mailbox-change hook (t87-e1): fired on every install of a mailbox
  // context (sign-in, sign-out, a switch either way) before any request for
  // the new mailbox is sent.
  const stopLinked = onLinkedMailboxChange(() => resetForSwitch());
  // The preference account is the second signal: every account change passes
  // through it too (suspend, then activate), so a path that changes the account
  // without installing a mailbox context still drops the keys.
  let bound: string | null | undefined;
  const stopObserving = observePreferenceAccount((next) => {
    const id = next?.id ?? null;
    if (bound === undefined) {
      bound = id;
      return;
    }
    if (id === bound) return;
    bound = id;
    resetForSwitch();
  });
  if (getOwner() !== null) {
    onCleanup(() => {
      stopLinked();
      stopObserving();
      resetForSwitch();
    });
  }

  /** Resolve (and cache) the crypto account id; `null` when none is available. */
  async function resolveAccount(): Promise<string | null> {
    const cur = accountId();
    if (cur !== null) return cur;
    const started = epoch;
    const session = await client.session();
    if (started !== epoch) throw mailboxChanged();
    const primary = session.primaryAccounts[CAP_CRYPTO];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    return acct;
  }

  async function loadKeys(): Promise<void> {
    const started = epoch;
    setKeysLoading(true);
    try {
      const acct = await resolveAccount();
      if (started !== epoch) throw mailboxChanged();
      if (acct === null) {
        setKeys([]);
        return;
      }
      const res = await client.jmap(keyQuery(acct));
      if (started !== epoch || accountId() !== acct) throw mailboxChanged();
      const got = responseFor<KeyGetResponse>(res, 'g');
      setKeys(tagged(acct, got.list));
    } catch (err) {
      // A load the switch overtook ends quietly; the new mailbox loads its own.
      if (started !== epoch) return;
      throw err;
    } finally {
      if (started === epoch) setKeysLoading(false);
    }
  }

  function keyOwnerAccount(key: CryptoKey): string | null {
    const owner = owners.get(key);
    return owner !== undefined && owner === accountId() ? owner : null;
  }

  async function unlockOwnKey(key: CryptoKey, passphrase: string): Promise<{ keyRef: KeyRef; accountId: string }> {
    const started = epoch;
    const acct = keyOwnerAccount(key);
    if (acct === null || !key.isOwn) throw new Error('This key does not belong to the open mailbox');
    const bundle = (await vault(acct).get(key.fingerprint))?.encryptedPrivateBundle ?? key.encryptedPrivateBackup;
    if (bundle === null) throw new Error('No private key is held for this key');
    if (started !== epoch) throw mailboxChanged();
    const keyRef = await worker.unlockKey({ encryptedPrivateBundle: bundle, passphrase });
    if (started !== epoch || accountId() !== acct) {
      // Unlocked for a mailbox that is no longer open: lock it straight away.
      await worker.lockKey({ keyRef }).catch(() => undefined);
      throw mailboxChanged();
    }
    issued.set(keyRef, acct);
    return { keyRef, accountId: acct };
  }

  async function lockKeyRef(keyRef: KeyRef): Promise<void> {
    issued.delete(keyRef);
    await worker.lockKey({ keyRef });
  }

  async function setKeyTrust(id: Id, trust: KeyTrust): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    setKeys(tagged(acct, keys().map((k) => (k.id === id ? { ...k, trust } : k))));
    await client.jmap({
      using: KEYS_USING,
      methodCalls: [['CryptoKey/setTrust', { accountId: acct, id, trust }, 's']],
    });
    ctx.broadcastChange?.();
  }

  /** Persist a public key (+ opaque backup) via `CryptoKey/set`; returns the row. */
  async function persistKey(acct: string, key: CryptoKey): Promise<CryptoKey> {
    // NEVER send plaintext private material — only the public key + the opaque
    // `encryptedPrivateBackup` blob (plan §1.2 / risk #4).
    const create: Record<string, unknown> = {
      kind: key.kind,
      isOwn: key.isOwn,
      addresses: key.addresses,
      fingerprint: key.fingerprint,
      keyId: key.keyId,
      algorithm: key.algorithm,
      createdAt: key.createdAt,
      expiresAt: key.expiresAt,
      publicKeyArmored: key.publicKeyArmored,
      certPem: key.certPem,
      trust: key.trust,
      autocrypt: key.autocrypt,
      source: key.source,
      hasPrivate: key.hasPrivate,
      encryptedPrivateBackup: key.encryptedPrivateBackup,
      verifiedAt: key.verifiedAt,
      keyHistory: key.keyHistory,
    };
    const res = await client.jmap({
      using: KEYS_USING,
      methodCalls: [['CryptoKey/set', { accountId: acct, create: { new: create } }, 's']],
    });
    const set = responseFor<KeySetResponse>(res, 's');
    const serverId = set.created?.['new']?.id;
    return serverId !== undefined ? { ...key, id: serverId } : key;
  }

  /** Insert (or replace) a key of `acct` in the local list, own keys first. */
  function upsert(acct: string, key: CryptoKey): void {
    if (acct !== accountId()) throw mailboxChanged();
    owners.set(key, acct);
    setKeys((cur) => {
      const rest = cur.filter((k) => k.id !== key.id && k.fingerprint !== key.fingerprint);
      return [...rest, key].sort((a, b) => Number(b.isOwn) - Number(a.isOwn));
    });
  }

  async function generateOwnKey(draft: OwnKeyDraft): Promise<CryptoKey> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    // Refuse a kind this slice does not generate (a cast, plain JS) before
    // anything reaches the worker, the vault or the server.
    if (draft.kind !== 'pgp' && draft.kind !== 'smime') throw new Error('unknown key kind');
    if (draft.kind === 'smime') return generateSmimeKey(acct, draft);
    const gen = await worker.generateKey({ kind: 'pgp', userId: draft.userId, passphrase: draft.passphrase });
    // An OpenPGP result carries armor and no certificate; anything else is not
    // filed as an OpenPGP key.
    if (gen.publicKeyArmored === undefined || gen.certPem !== undefined) {
      throw new Error('the crypto worker did not return an OpenPGP key');
    }
    const address = addressOf(draft.userId);
    const now = new Date().toISOString();
    await vault(acct).put({
      fingerprint: gen.fingerprint,
      kind: 'pgp',
      encryptedPrivateBundle: gen.encryptedPrivateBundle,
      addresses: [address],
    });
    setVaulted((s) => new Set(s).add(gen.fingerprint));
    // Own keys are trusted by construction (we hold the private half).
    const key: CryptoKey = {
      id: `own-${gen.fingerprint}`,
      kind: 'pgp',
      isOwn: true,
      addresses: [address],
      fingerprint: gen.fingerprint,
      keyId: gen.keyId,
      algorithm: 'ed25519',
      createdAt: now,
      expiresAt: null,
      publicKeyArmored: gen.publicKeyArmored,
      certPem: null,
      trust: 'verified',
      autocrypt: true,
      source: 'generated',
      hasPrivate: true,
      encryptedPrivateBackup: gen.encryptedPrivateBundle,
      verifiedAt: now,
      keyHistory: [{ fingerprint: gen.fingerprint, seenAt: now }],
    };
    const stored = await persistKey(acct, key);
    upsert(acct, stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', 'PGP key generated');
    return stored;
  }

  /**
   * Generate an S/MIME key. Everything the row says about the certificate —
   * addresses, algorithm, expiry — is what the worker read back from the
   * certificate it made, not what this function assumes it asked for.
   */
  async function generateSmimeKey(acct: string, draft: OwnKeyDraft): Promise<CryptoKey> {
    const gen = await worker.generateKey({ kind: 'smime', userId: draft.userId, passphrase: draft.passphrase });
    // An S/MIME result is a certificate and no OpenPGP armor. Checked before
    // anything is stored, so one kind is never filed as the other.
    if (
      gen.certPem === undefined ||
      !gen.certPem.includes('-----BEGIN CERTIFICATE-----') ||
      gen.publicKeyArmored !== undefined ||
      gen.algorithm === undefined ||
      gen.addresses === undefined ||
      gen.expiresAt === undefined
    ) {
      throw new Error('the crypto worker did not return an S/MIME certificate');
    }
    const now = new Date().toISOString();
    await vault(acct).put({
      fingerprint: gen.fingerprint,
      kind: 'smime',
      encryptedPrivateBundle: gen.encryptedPrivateBundle,
      addresses: gen.addresses,
    });
    setVaulted((s) => new Set(s).add(gen.fingerprint));
    const key: CryptoKey = {
      id: `own-${gen.fingerprint}`,
      kind: 'smime',
      isOwn: true,
      addresses: gen.addresses,
      fingerprint: gen.fingerprint,
      keyId: gen.keyId,
      algorithm: gen.algorithm,
      createdAt: now,
      expiresAt: gen.expiresAt,
      publicKeyArmored: null,
      certPem: gen.certPem,
      // "verified" here means what it means for every own key: we hold the
      // private half. It says nothing about a certificate authority.
      trust: 'verified',
      autocrypt: false,
      source: 'generated',
      hasPrivate: true,
      encryptedPrivateBackup: gen.encryptedPrivateBundle,
      verifiedAt: now,
      keyHistory: [{ fingerprint: gen.fingerprint, seenAt: now }],
    };
    const stored = await persistKey(acct, key);
    upsert(acct, stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', 'S/MIME key and self-signed certificate generated');
    return stored;
  }

  async function previewArmoredKey(armored: string, passphrase?: string): Promise<ImportPreview> {
    const res = await worker.importArmored(passphrase === undefined ? { armored } : { armored, passphrase });
    return { key: res.key, encryptedPrivateBundle: res.encryptedPrivateBundle ?? null };
  }

  async function previewPkcs12Key(p12Bytes: Uint8Array, password: string): Promise<ImportPreview> {
    const res = await worker.importPkcs12({ p12Bytes, password });
    const now = new Date().toISOString();
    const key: CryptoKey = {
      id: `imported-${res.fingerprint}`,
      kind: 'smime',
      isOwn: true,
      addresses: res.addresses,
      fingerprint: res.fingerprint,
      keyId: res.fingerprint.slice(0, 16),
      algorithm: res.algorithm,
      createdAt: now,
      expiresAt: res.expiresAt,
      publicKeyArmored: null,
      certPem: res.certPem,
      trust: 'verified',
      autocrypt: false,
      source: 'pkcs12',
      hasPrivate: true,
      encryptedPrivateBackup: res.encryptedPrivateBundle,
      verifiedAt: now,
      keyHistory: [{ fingerprint: res.fingerprint, seenAt: now }],
    };
    return { key, encryptedPrivateBundle: res.encryptedPrivateBundle };
  }

  async function commitImport(preview: ImportPreview): Promise<CryptoKey> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    const { key, encryptedPrivateBundle } = preview;
    const hasPrivate = encryptedPrivateBundle !== null;
    // The same key again (for instance a PKCS#12 exported from here): keep the
    // row that exists instead of storing a second one for one fingerprint.
    const held = keys().find((k) => k.fingerprint === key.fingerprint && k.kind === key.kind && k.hasPrivate);
    if (held !== undefined) {
      ctx.showToast('info', 'This key is already in your keys; nothing was added');
      return held;
    }
    if (encryptedPrivateBundle !== null) {
      await vault(acct).put({
        fingerprint: key.fingerprint,
        kind: key.kind,
        encryptedPrivateBundle,
        addresses: key.addresses,
      });
      setVaulted((s) => new Set(s).add(key.fingerprint));
    }
    const source: KeySource = key.source === 'pkcs12' ? 'pkcs12' : 'imported';
    const row: CryptoKey = { ...key, isOwn: hasPrivate, hasPrivate, source, encryptedPrivateBackup: encryptedPrivateBundle };
    const stored = await persistKey(acct, row);
    upsert(acct, stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', 'Key imported');
    return stored;
  }

  async function exportKeyBackup(fingerprint: string): Promise<string> {
    const acct = accountId();
    const entry = acct === null ? null : await vault(acct).get(fingerprint);
    if (entry === null) throw new Error('no private key held for this fingerprint');
    const res = await worker.exportBackup({ encryptedPrivateBundle: entry.encryptedPrivateBundle, kind: entry.kind });
    return res.autocryptSetupMessage;
  }

  /** An own S/MIME key with its certificate and wrapped private key, or a throw. */
  async function ownSmimeKey(id: Id): Promise<{ key: CryptoKey; certPem: string; bundle: string }> {
    const key = keys().find((k) => k.id === id);
    if (key === undefined || key.kind !== 'smime' || !key.isOwn || key.certPem === null) {
      throw new Error('not an own S/MIME key');
    }
    // The vault is per-session; after a reload the same wrapped bundle is the
    // row's opaque backup.
    const acct = keyOwnerAccount(key);
    if (acct === null) throw new Error('not an own S/MIME key');
    const bundle = (await vault(acct).get(key.fingerprint))?.encryptedPrivateBundle ?? key.encryptedPrivateBackup;
    if (bundle === null) throw new Error('no private key held for this certificate');
    return { key, certPem: key.certPem, bundle };
  }

  async function exportSmimePkcs12(id: Id, passphrase: string): Promise<Uint8Array> {
    const { certPem, bundle } = await ownSmimeKey(id);
    const res = await worker.exportPkcs12({ certPem, encryptedPrivateBundle: bundle, passphrase });
    return Uint8Array.from(atob(res.p12Base64), (c) => c.charCodeAt(0));
  }

  async function smimeCertificateRequest(id: Id, passphrase: string): Promise<string> {
    const { certPem, bundle } = await ownSmimeKey(id);
    const res = await worker.certificateRequest({ certPem, encryptedPrivateBundle: bundle, passphrase });
    return res.csrPem;
  }

  async function attachIssuedCertificate(id: Id, certBytes: Uint8Array, passphrase: string): Promise<CryptoKey> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    const { key: old, bundle } = await ownSmimeKey(id);
    // The worker rejects a certificate whose public key is not this key's.
    const res = await worker.attachIssuedCert({ certBytes, encryptedPrivateBundle: bundle, passphrase });
    if (res.fingerprint === old.fingerprint) throw new Error('this is the certificate already held for the key');
    const now = new Date().toISOString();
    await vault(acct).put({
      fingerprint: res.fingerprint,
      kind: 'smime',
      encryptedPrivateBundle: bundle,
      addresses: res.addresses,
    });
    setVaulted((s) => new Set(s).add(res.fingerprint));
    // A key's fingerprint is its certificate's, so a new certificate is a new
    // row; the old row is removed once the new one is stored.
    const row: CryptoKey = {
      ...old,
      id: `own-${res.fingerprint}`,
      addresses: res.addresses,
      fingerprint: res.fingerprint,
      keyId: res.fingerprint.slice(0, 16),
      algorithm: res.algorithm,
      expiresAt: res.expiresAt,
      certPem: res.certPem,
      source: 'imported',
      encryptedPrivateBackup: bundle,
      keyHistory: [...old.keyHistory, { fingerprint: res.fingerprint, seenAt: now }],
    };
    const stored = await persistKey(acct, row);
    await client.jmap({
      using: KEYS_USING,
      methodCalls: [['CryptoKey/set', { accountId: acct, destroy: [old.id] }, 'd']],
    });
    await vault(acct).remove(old.fingerprint);
    setVaulted((s) => {
      const next = new Set(s);
      next.delete(old.fingerprint);
      return next;
    });
    setKeys((cur) => cur.filter((k) => k.id !== old.id));
    upsert(acct, stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', 'Certificate replaced');
    return stored;
  }

  async function lookupContactKey(address: string, sources: KeyLookupSource[]): Promise<CryptoKey[]> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    const res = await client.jmap({
      using: KEYS_USING,
      methodCalls: [['CryptoKey/lookup', { accountId: acct, address, sources }, 'l']],
    });
    const got = responseFor<KeyLookupResponse>(res, 'l');
    // Looked-up keys are contact keys (public-only) regardless of the mock's own
    // flag; TOFU trust starts unverified until the user confirms out-of-band.
    const found = got.list.map((k): CryptoKey => ({ ...k, isOwn: false, hasPrivate: false }));
    for (const k of found) upsert(acct, k);
    if (found.length > 0) ctx.broadcastChange?.();
    return found;
  }

  async function associateKeyWithContact(contactId: Id, key: CryptoKey): Promise<void> {
    const session = await client.session();
    const acct = session.primaryAccounts[CAP_CONTACTS] ?? Object.keys(session.accounts)[0];
    if (acct === undefined) throw new Error('no contacts account available');
    const patch = key.kind === 'pgp' ? { pgpKey: key.publicKeyArmored } : { smimeCert: key.certPem };
    await client.jmap({
      using: [CAP_CORE, CAP_CONTACTS],
      methodCalls: [['ContactCard/set', { accountId: acct, update: { [contactId]: patch } }, 'a']],
    });
    ctx.broadcastChange?.();
    ctx.showToast('success', 'Key associated with contact');
  }

  return {
    keys,
    keysLoading,
    ownKeys,
    contactKeys,
    loadKeys,
    setKeyTrust,
    generateOwnKey,
    previewArmoredKey,
    previewPkcs12Key,
    commitImport,
    exportKeyBackup,
    exportSmimePkcs12,
    smimeCertificateRequest,
    attachIssuedCertificate,
    lookupContactKey,
    associateKeyWithContact,
    hasVaultedKey,
    keysAccountId: accountId,
    keyOwnerAccount,
    unlockOwnKey,
    lockKeyRef,
    keysEpoch,
  };
}
