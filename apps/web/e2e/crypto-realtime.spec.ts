import { test, expect } from '@playwright/test';
import {
  engineLogin,
  gotoKeys,
  uid,
  CAP_CORE,
  CAP_CRYPTO,
  cryptoAccountId,
} from './crypto-helpers.ts';

/**
 * V4 realtime live E2E (plan §3 e10 item 8 / §2.2): a crypto `StateChange` updates
 * an open key list WITHOUT a manual refresh. The Reader shell reloads the keyring on
 * any pushed crypto change while the keys surface is open (Mailbox `onRealtimeChange`).
 *
 * The change is originated by a raw `CryptoKey/set` issued OUT-OF-BAND from the app's
 * own key UI (over `page.request`, which shares the browser session), then the open
 * key list must pick it up via the push — not via the action that made the change.
 * This exercises the real wire end-to-end: engine `broadcast_state` → mw-server WS/SSE
 * `StateChange` (CryptoKey key) → the open browser reloads the list. It does NOT use
 * the WASM worker, so it is independent of the crypto-worker CSP path.
 *
 * NOTE (engine mode): the account id is minted per-login, so a genuinely separate
 * login is a DIFFERENT account and its broadcast would not reach this page. We
 * therefore drive the out-of-band set over the page's own session (same account) —
 * still a push-delivered list update the app did not trigger through its keys slice.
 */
test('Realtime: an out-of-band CryptoKey change appears in the open key list via push', async ({
  page,
}) => {
  const token = uid();
  const newAddr = `realtime-${token}@example.org`;

  await engineLogin(page);
  await gotoKeys(page);

  // The new key is not present yet.
  const ownKeys = page.getByRole('list', { name: 'Your keys' });
  await expect(ownKeys.getByText(newAddr)).toHaveCount(0);

  // Add a public key via a raw CryptoKey/set over the page's session — the mutation
  // that fires the crypto StateChange the open list must react to.
  const acct = await cryptoAccountId(page.request);
  const now = new Date().toISOString();
  const create = {
    // JMAP servers assign the id; the engine also accepts a present-but-empty id
    // (it mints one). Sent explicitly so serde does not reject a missing field.
    id: '',
    kind: 'pgp',
    isOwn: true,
    addresses: [newAddr],
    fingerprint: `RTFPR${token}`.toUpperCase().padEnd(40, '0'),
    keyId: `RT${token}`.toUpperCase(),
    algorithm: 'ed25519',
    createdAt: now,
    expiresAt: null,
    publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nRT\n-----END PGP PUBLIC KEY BLOCK-----',
    certPem: null,
    trust: 'verified',
    autocrypt: true,
    source: 'generated',
    hasPrivate: false,
    encryptedPrivateBackup: null,
    verifiedAt: now,
    keyHistory: [],
  };
  const setRes = await page.request.post('/jmap/api', {
    headers: { 'content-type': 'application/json' },
    data: {
      using: [CAP_CORE, CAP_CRYPTO],
      methodCalls: [['CryptoKey/set', { accountId: acct, create: { new: create } }, 's']],
    },
  });
  expect(setRes.ok()).toBeTruthy();
  const created = ((await setRes.json()) as { methodResponses: [string, { created?: Record<string, unknown> }, string][] })
    .methodResponses[0][1].created;
  expect(created && Object.keys(created).length, 'CryptoKey/set created the key').toBeTruthy();

  // The open key list picks it up via the realtime push — no manual reload.
  await expect(ownKeys.getByText(newAddr)).toBeVisible({ timeout: 20_000 });
});
