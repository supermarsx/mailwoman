// Keys store slice (plan §2.5, §3 e0 stub → e2 fills). Owns the `CryptoKey/*`
// surface for the web client: the own + contact/harvested key list, generate,
// import (armored + PKCS#12), backup (Autocrypt Setup Message), trust/verify,
// WKD/VKS/harvest lookup, and per-contact key association. Disjoint file — no
// `store.ts` collision with the other slices (same discipline as the V2/V3 slices).
//
// PRIVACY (plan §1.2 / risk #4): private-key generation/import runs in the crypto
// WORKER (`crypto/index.ts`), and the wrapped private bundle lives ONLY in the
// client `KeyVault`; this slice + the server hold PUBLIC key metadata plus the
// OPAQUE `encryptedPrivateBackup` (which the server never decrypts). e2 fills the
// full flows against the mock + the worker STUB; e8 swaps to the real engine +
// wasm worker (by flipping `getCryptoWorker()` — this slice is unchanged then).

import { createSignal, type Accessor } from 'solid-js';
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
import type { SliceContext } from './context.ts';

const KEYS_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

/** A source the keyring can look a contact key up from (§2.2 `CryptoKey/lookup`). */
export type KeyLookupSource = 'wkd' | 'vks' | 'autocrypt' | 'harvested';

/** The fields a new own-key generation collects from the UI. */
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
  /** Look a contact key up over WKD/VKS/autocrypt/harvest (consent-gated in UI). */
  lookupContactKey(address: string, sources: KeyLookupSource[]): Promise<CryptoKey[]>;
  /** Write a key onto a V3 contact card (`pgpKey`/`smimeCert`), populating it. */
  associateKeyWithContact(contactId: Id, key: CryptoKey): Promise<void>;
  /** Whether a wrapped private bundle for `fingerprint` is held in the vault. */
  hasVaultedKey(fingerprint: string): boolean;
}

export function createKeysSlice(ctx: SliceContext): KeysSlice {
  const client = ctx.client;
  const worker = getCryptoWorker();
  const vault: KeyVault = createInMemoryVault();
  const [keys, setKeys] = createSignal<CryptoKey[]>([]);
  const [keysLoading, setKeysLoading] = createSignal(false);
  const [accountId, setAccountId] = createSignal<string | null>(null);
  const [vaulted, setVaulted] = createSignal<Set<string>>(new Set());

  const ownKeys = (): CryptoKey[] => keys().filter((k) => k.isOwn);
  const contactKeys = (): CryptoKey[] => keys().filter((k) => !k.isOwn);
  const hasVaultedKey = (fingerprint: string): boolean => vaulted().has(fingerprint);

  /** Resolve (and cache) the crypto account id; `null` when none is available. */
  async function resolveAccount(): Promise<string | null> {
    const cur = accountId();
    if (cur !== null) return cur;
    const session = await client.session();
    const primary = session.primaryAccounts[CAP_CRYPTO];
    const acct = primary ?? Object.keys(session.accounts)[0] ?? null;
    setAccountId(acct);
    return acct;
  }

  async function loadKeys(): Promise<void> {
    setKeysLoading(true);
    try {
      const acct = await resolveAccount();
      if (acct === null) {
        setKeys([]);
        return;
      }
      const res = await client.jmap(keyQuery(acct));
      const got = responseFor<KeyGetResponse>(res, 'g');
      setKeys(got.list);
    } finally {
      setKeysLoading(false);
    }
  }

  async function setKeyTrust(id: Id, trust: KeyTrust): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    setKeys(keys().map((k) => (k.id === id ? { ...k, trust } : k)));
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

  /** Insert (or replace) a key in the local list, keeping own keys first. */
  function upsert(key: CryptoKey): void {
    setKeys((cur) => {
      const rest = cur.filter((k) => k.id !== key.id && k.fingerprint !== key.fingerprint);
      return [...rest, key].sort((a, b) => Number(b.isOwn) - Number(a.isOwn));
    });
  }

  async function generateOwnKey(draft: OwnKeyDraft): Promise<CryptoKey> {
    const acct = await resolveAccount();
    if (acct === null) throw new Error('no account available for keys');
    const gen = await worker.generateKey({ kind: draft.kind, userId: draft.userId, passphrase: draft.passphrase });
    const address = addressOf(draft.userId);
    const now = new Date().toISOString();
    await vault.put({
      fingerprint: gen.fingerprint,
      kind: draft.kind,
      encryptedPrivateBundle: gen.encryptedPrivateBundle,
      addresses: [address],
    });
    setVaulted((s) => new Set(s).add(gen.fingerprint));
    // Own keys are trusted by construction (we hold the private half).
    const key: CryptoKey = {
      id: `own-${gen.fingerprint}`,
      kind: draft.kind,
      isOwn: true,
      addresses: [address],
      fingerprint: gen.fingerprint,
      keyId: gen.keyId,
      algorithm: draft.kind === 'pgp' ? 'ed25519' : 'ecdsa-p256',
      createdAt: now,
      expiresAt: null,
      publicKeyArmored: draft.kind === 'pgp' ? gen.publicKeyArmored : null,
      certPem: draft.kind === 'smime' ? gen.publicKeyArmored : null,
      trust: 'verified',
      autocrypt: draft.kind === 'pgp',
      source: 'generated',
      hasPrivate: true,
      encryptedPrivateBackup: gen.encryptedPrivateBundle,
      verifiedAt: now,
      keyHistory: [{ fingerprint: gen.fingerprint, seenAt: now }],
    };
    const stored = await persistKey(acct, key);
    upsert(stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', `${draft.kind.toUpperCase()} key generated`);
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
      addresses: [],
      fingerprint: res.fingerprint,
      keyId: res.fingerprint.slice(0, 16),
      algorithm: 'rsa',
      createdAt: now,
      expiresAt: null,
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
    if (encryptedPrivateBundle !== null) {
      await vault.put({
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
    upsert(stored);
    ctx.broadcastChange?.();
    ctx.showToast('success', 'Key imported');
    return stored;
  }

  async function exportKeyBackup(fingerprint: string): Promise<string> {
    const entry = await vault.get(fingerprint);
    if (entry === null) throw new Error('no private key held for this fingerprint');
    const res = await worker.exportBackup({ encryptedPrivateBundle: entry.encryptedPrivateBundle, kind: entry.kind });
    return res.autocryptSetupMessage;
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
    for (const k of found) upsert(k);
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
    lookupContactKey,
    associateKeyWithContact,
    hasVaultedKey,
  };
}
