import { test, expect } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { engineLogin, gotoKeys } from './crypto-helpers.ts';

/**
 * V4 S/MIME live E2E (plan §3 e10 item 3 / SPEC §8.2): import a PKCS#12 bundle
 * through the REAL UI + REAL WASM worker (`importPkcs12`), producing an own S/MIME
 * key on this device. The private key material is parsed IN the crypto worker and
 * wrapped into the client vault; only the public cert + the opaque backup reach the
 * server (plan §1.2).
 *
 * Scope note (plan §3 e10, "if PKCS#12 import UX is thin, assert what's wired +
 * note"): a full in-browser S/MIME sign→verify→3-state-badge round-trip is thin —
 * sign-on-send is a documented e8 follow-up (the toggle reports it but does not yet
 * apply it to the ciphertext), and signature VERIFY is the server-side 3-state badge
 * proven by crypto-security.spec + the mw-crypto interop fixtures. So this spec
 * asserts the wired path: PKCS#12 import via the real worker yields an S/MIME key row.
 *
 * Requires the crypto worker WASM to load in the browser (server CSP `script-src`
 * carries `'wasm-unsafe-eval'`) and `CryptoKey/set` to persist a create with no id —
 * both on master (commits 5c7752d, 0987105).
 */
const P12 = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '../../../fixtures/crypto/smime/alice.p12',
);
const P12_PASSWORD = 'test'; // fixtures/crypto/README.md

test('S/MIME: importing a PKCS#12 bundle adds an S/MIME key (real WASM parse)', async ({ page }) => {
  test.setTimeout(60_000);
  await engineLogin(page);
  await gotoKeys(page);

  await page.getByRole('button', { name: 'Import key', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Import a key' });
  await expect(dialog).toBeVisible();

  // Switch to the PKCS#12 tab, provide the fixture + its password.
  await dialog.getByRole('tab', { name: 'PKCS#12 (S/MIME)' }).click();
  await dialog.getByLabel('PKCS#12 file').setInputFiles(P12);
  await dialog.getByLabel('PKCS#12 password', { exact: true }).fill(P12_PASSWORD);

  // Preview parses the bundle in the worker and shows the fingerprint.
  await dialog.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(dialog.getByRole('group', { name: 'Import preview' })).toBeVisible({ timeout: 30_000 });
  await expect(dialog.getByLabel('Preview fingerprint')).toBeVisible();

  // Commit the import → an own S/MIME key row appears (vaulted private + public cert).
  await dialog.getByRole('button', { name: 'Import', exact: true }).click();
  await expect(dialog).toBeHidden({ timeout: 30_000 });

  const ownKeys = page.getByRole('list', { name: 'Your keys' });
  await expect(ownKeys.getByText(/SMIME/i).first()).toBeVisible({ timeout: 30_000 });
});
