import { test, expect } from '@playwright/test';
import { engineLogin, gotoKeys } from './crypto-helpers.ts';

/**
 * V4 key-management reachability gate (plan §3 e10 / DoD "mounted + wired"): the
 * key-management module boots REACHABLE from the app-shell nav rail and is engine-
 * backed — the explicit mount step (e8). The generate + import affordances open. The
 * actual browser-side keygen/import/decrypt (real WASM) is exercised by the
 * crypto-pgp / crypto-smime specs; this spec only proves the module mounts + the UI
 * is live, so it holds even where the WASM worker path is gated.
 */
test('Key management is reachable from the shell nav and its dialogs open', async ({ page }) => {
  await engineLogin(page);

  // Reachable from the "Apps" nav rail beside the PIM modules.
  await expect(page.getByTestId('nav-keys')).toBeVisible();
  await gotoKeys(page);

  // The module's own controls render (own-key + contact-key generate/import/lookup).
  await expect(page.getByRole('button', { name: 'Generate key', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Import key', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Keys & certificates' })).toBeVisible();

  // The generate dialog opens with its OpenPGP/S-MIME fields (mounted + interactive).
  await page.getByRole('button', { name: 'Generate key', exact: true }).click();
  const genDialog = page.getByRole('dialog', { name: 'Generate a key' });
  await expect(genDialog).toBeVisible();
  await expect(genDialog.getByLabel('Key type')).toBeVisible();
  await expect(genDialog.getByLabel('Email', { exact: true })).toBeVisible();
  await expect(genDialog.getByLabel('Key passphrase', { exact: true })).toBeVisible();
  await genDialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(genDialog).toBeHidden();

  // The import dialog opens with the armored + PKCS#12 tabs.
  await page.getByRole('button', { name: 'Import key', exact: true }).click();
  const impDialog = page.getByRole('dialog', { name: 'Import a key' });
  await expect(impDialog).toBeVisible();
  await expect(impDialog.getByRole('tab', { name: 'Armored (PGP)' })).toBeVisible();
  await expect(impDialog.getByRole('tab', { name: 'PKCS#12 (S/MIME)' })).toBeVisible();
});
