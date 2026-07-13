// Keys store slice (plan §2.5, §3 e0 stub → e2 fills). Owns the `CryptoKey/*`
// surface for the web client: the own + contact/harvested key list, generate,
// import, trust/verify, and WKD/VKS lookup. Disjoint file — no `store.ts`
// collision with the other slices (same discipline as the V2/V3 slices).
//
// PRIVACY (plan §1.2 / risk #4): private-key generation/import runs in the crypto
// WORKER (`crypto/index.ts`); this slice only ever holds PUBLIC key metadata + the
// opaque `encryptedPrivateBackup`. e0 ships this stub (mock-backed, worker-stub);
// e2 fills the full key-management flows; e8 swaps to the real engine + wasm worker.

import { createSignal, type Accessor } from 'solid-js';
import { CAP_CORE, type Id, type JmapRequest } from '../../api/jmap-types.ts';
import { responseFor } from '../../api/jmap.ts';
import { CAP_CRYPTO, CAP_SECURITY, type CryptoKey, type KeyTrust } from '../../api/crypto-types.ts';
import type { SliceContext } from './context.ts';

const KEYS_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

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
}

export function createKeysSlice(ctx: SliceContext): KeysSlice {
  const client = ctx.client;
  const [keys, setKeys] = createSignal<CryptoKey[]>([]);
  const [keysLoading, setKeysLoading] = createSignal(false);
  const [accountId, setAccountId] = createSignal<string | null>(null);

  const ownKeys = (): CryptoKey[] => keys().filter((k) => k.isOwn);
  const contactKeys = (): CryptoKey[] => keys().filter((k) => !k.isOwn);

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

  return { keys, keysLoading, ownKeys, contactKeys, loadKeys, setKeyTrust };
}
