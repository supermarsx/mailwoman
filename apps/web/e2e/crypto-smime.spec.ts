import { test, expect } from '@playwright/test';
import { X509Certificate } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { cryptoAccountId, engineLogin, gotoKeys, jmapCall, uid } from './crypto-helpers.ts';

/**
 * S/MIME key management, live: the REAL UI and the REAL WASM worker against the
 * engine-mode server.
 *
 * What these two tests cover, and nothing more:
 *  1. importing a PKCS#12 bundle (`importPkcs12` in the worker) yields an own
 *     S/MIME key row;
 *  2. generating an S/MIME key in the dialog yields a real X.509 certificate
 *     (parsed here by Node's `X509Certificate`, not by the app), which exports as
 *     a `.p12` file that the import dialog reads back as the same certificate.
 *
 * What they do NOT cover: signing, verifying, encrypting or decrypting S/MIME
 * mail in the app. Compose and the reader use OpenPGP keys only; there is no
 * S/MIME signature badge test anywhere in this suite. The CMS operations
 * themselves are unit-tested against openssl in `crates/mw-crypto/tests/smime.rs`
 * and `smime_generate.rs`.
 *
 * Private keys are generated and parsed IN the crypto worker and wrapped into the
 * client vault; the server receives the certificate and the passphrase-wrapped
 * bundle (plan §1.2). Requires the worker WASM to load in the browser (server CSP
 * `script-src` carries `'wasm-unsafe-eval'`) and `CryptoKey/set` to persist a
 * create with no id.
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

interface StoredKey {
  id: string;
  kind: string;
  addresses: string[];
  fingerprint: string;
  algorithm: string;
  expiresAt: string | null;
  publicKeyArmored: string | null;
  certPem: string | null;
  source: string;
}

test('S/MIME: a generated key is a real certificate, exports as .p12 and imports back', async ({ page }, testInfo) => {
  // RSA key generation runs in the browser; the dialog says it can take a minute.
  test.setTimeout(300_000);
  const address = `smime-${uid()}@example.org`;
  const passphrase = `pw-${uid()}`;

  await engineLogin(page);
  await gotoKeys(page);

  // Generate through the dialog with S/MIME chosen.
  await page.getByRole('button', { name: 'Generate key', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Generate a key' });
  await dialog.getByLabel('Key type').selectOption('smime');
  await expect(dialog.getByText(/The certificate is self-signed/)).toBeVisible();
  await dialog.getByLabel('Name', { exact: true }).fill('E2E S/MIME');
  await dialog.getByLabel('Email', { exact: true }).fill(address);
  await dialog.getByLabel('Key passphrase', { exact: true }).fill(passphrase);
  const started = Date.now();
  await dialog.getByRole('button', { name: 'Generate', exact: true }).click();
  await expect(dialog).toBeHidden({ timeout: 240_000 });
  testInfo.annotations.push({ type: 'smime-keygen-ms', description: String(Date.now() - started) });
  console.log(`S/MIME key generation in the browser (click to dialog closed): ${Date.now() - started} ms`);

  const ownKeys = page.getByRole('list', { name: 'Your keys' });
  await expect(ownKeys.getByText(address)).toHaveCount(1);

  // What the server now holds for that address — judged by Node's X.509 parser.
  const accountId = await cryptoAccountId(page.request);
  const listStored = async (): Promise<StoredKey[]> => {
    const got = await jmapCall<{ list: StoredKey[] }>(
      page.request,
      [
        ['CryptoKey/query', { accountId }, 'q'],
        ['CryptoKey/get', { accountId, '#ids': { resultOf: 'q', name: 'CryptoKey/query', path: '/ids' } }, 'g'],
      ],
      'g',
    );
    return got.list.filter((k) => k.addresses.includes(address));
  };
  const stored = await listStored();
  expect(stored).toHaveLength(1);
  const row = stored[0]!;
  expect(row.kind).toBe('smime');
  expect(row.source).toBe('generated');
  expect(row.publicKeyArmored).toBeNull();
  expect(row.certPem).not.toBeNull();
  const cert = new X509Certificate(row.certPem!);
  expect(cert.publicKey.asymmetricKeyType).toBe('rsa');
  expect(row.algorithm).toBe(`rsa-${cert.publicKey.asymmetricKeyDetails?.modulusLength}`);
  expect(row.algorithm).toBe('rsa-3072');
  expect(cert.subjectAltName).toBe(`email:${address}`);
  expect(cert.verify(cert.publicKey)).toBe(true); // self-signed
  expect(cert.ca).toBe(false);
  expect(cert.keyUsage).toEqual(['1.3.6.1.5.5.7.3.4']); // id-kp-emailProtection
  expect(row.fingerprint).toBe(cert.fingerprint256.replaceAll(':', ''));
  expect(new Date(row.expiresAt!).getTime()).toBe(new Date(cert.validTo).getTime());

  // The detail pane says what it is and offers the export.
  await ownKeys.getByText(address).click();
  const section = page.getByRole('group', { name: 'S/MIME certificate' });
  await expect(section.getByText(/This certificate is self-signed/)).toBeVisible();
  await expect(page.getByRole('button', { name: 'Export backup' })).toHaveCount(0);
  const exportButton = section.getByRole('button', { name: 'Export as PKCS#12 (.p12)' });
  await expect(exportButton).toBeDisabled();

  // A wrong passphrase exports nothing.
  await section.getByLabel('Passphrase for this S/MIME key').fill('not the passphrase');
  await exportButton.click();
  await expect(section.getByRole('alert')).toContainText('Check the passphrase');

  // The right one downloads the .p12 and the request.
  await section.getByLabel('Passphrase for this S/MIME key').fill(passphrase);
  const p12Download = page.waitForEvent('download');
  await exportButton.click();
  const p12 = await p12Download;
  expect(p12.suggestedFilename()).toMatch(/\.p12$/);
  const p12Path = testInfo.outputPath('generated.p12');
  await p12.saveAs(p12Path);

  const csrDownload = page.waitForEvent('download');
  await section.getByRole('button', { name: 'Download certificate request (.csr)' }).click();
  const csr = await csrDownload;
  expect(csr.suggestedFilename()).toMatch(/\.csr$/);
  const csrPath = testInfo.outputPath('generated.csr');
  await csr.saveAs(csrPath);
  const { readFileSync } = await import('node:fs');
  expect(readFileSync(csrPath, 'utf8')).toMatch(/^-----BEGIN CERTIFICATE REQUEST-----\n/);

  // Import the exported file back through the existing import dialog.
  await page.getByRole('button', { name: 'Import key', exact: true }).click();
  const importDialog = page.getByRole('dialog', { name: 'Import a key' });
  await importDialog.getByRole('tab', { name: 'PKCS#12 (S/MIME)' }).click();
  await importDialog.getByLabel('PKCS#12 file').setInputFiles(p12Path);

  // Precondition for "it reads back": the wrong password does not open it.
  await importDialog.getByLabel('PKCS#12 password', { exact: true }).fill('not the passphrase');
  await importDialog.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(importDialog.getByRole('button', { name: 'Preview', exact: true })).toBeEnabled({ timeout: 60_000 });
  await expect(importDialog.getByRole('group', { name: 'Import preview' })).toHaveCount(0);

  await importDialog.getByLabel('PKCS#12 password', { exact: true }).fill(passphrase);
  await importDialog.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(importDialog.getByRole('group', { name: 'Import preview' })).toBeVisible({ timeout: 60_000 });
  await importDialog.getByRole('button', { name: 'Import', exact: true }).click();
  await expect(importDialog).toBeHidden({ timeout: 60_000 });

  // Still one certificate for the typed address, in the list and on the server:
  // the import was recognised as the key already held (same fingerprint), so no
  // second row was stored.
  await expect(ownKeys.getByText(address)).toHaveCount(1);
  const after = await listStored();
  expect(after).toHaveLength(1);
  expect(after[0]!.id).toBe(row.id);
  expect(after[0]!.fingerprint).toBe(row.fingerprint);
});
