import { test, expect } from '@playwright/test';
import {
  engineLogin,
  ENGINE_CREDS,
  gotoKeys,
  generatePgpKey,
  uid,
  waitForInboxMessage,
  openMessage,
} from './crypto-helpers.ts';

/**
 * V4 headline live E2E (plan §3 e10, DoD OpenPGP): a full browser-side PGP
 * round-trip through the REAL UI + REAL WASM worker + REAL engine.
 *
 *   generate a key in the browser (real WASM keygen)
 *     → compose to self, the E2EE/TLS/mixed banner goes live from CryptoKey/lookup
 *     → toggle encrypt (real WASM encrypts the body client-side)
 *     → Send (the armored ciphertext is submitted over SMTP → loops back to the
 *       IMAP inbox through mw-engine)
 *     → open the received message → Decrypt with the passphrase (real WASM decrypt,
 *       using the key's opaque encryptedPrivateBackup) → the plaintext renders.
 *
 * The encrypted plaintext is deliberately HTML carrying a <script> + an inline
 * handler: the decrypted body is sanitized IN THE CRYPTO WORKER (mw-sanitize wasm,
 * §1.3) before it reaches the sandboxed iframe, so the round-trip ALSO proves the
 * decrypted-HTML sanitization (plan §3 e10 item 2) in one send.
 *
 * Serial + single test: exactly one own PGP key is minted for the self address, so
 * both the encrypt (CryptoKey/lookup, newest-own-first) and the decrypt (first own
 * key with a private backup) resolve to the SAME keypair deterministically.
 */
test.describe.configure({ mode: 'serial' });

test('PGP: generate → encrypt → send → decrypt-on-receipt (round-trip + in-worker sanitize)', async ({
  page,
}) => {
  test.setTimeout(150_000); // real WASM keygen + SMTP undo-hold + IMAP loopback

  const token = uid();
  const subject = `PGP round-trip ${token}`;
  const marker = `SECRET_${token}`;
  const boldMarker = `BOLD_${token}`;
  const passphrase = `e2e-pgp-pass-${token}`;
  // The plaintext is HTML with a hostile <script> + inline handler that the
  // in-worker sanitizer must strip on decrypt, plus two visible markers.
  const plaintextHtml =
    `<p>${marker}</p>` +
    `<script>window.__pwned_${token}=1</script>` +
    `<b onclick="window.__pwned_${token}=2">${boldMarker}</b>`;

  await engineLogin(page);

  // 1) Generate a real OpenPGP key in the browser for our own address.
  await gotoKeys(page);
  await generatePgpKey(page, { email: ENGINE_CREDS.selfAddress, passphrase, name: 'Test User' });

  // 2) Compose to self; the crypto banner must detect the recipient key (e2ee).
  await page.getByRole('button', { name: 'Compose', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(plaintextHtml);

  const banner = dialog.locator('[data-testid="compose-crypto-banner"]');
  await expect(banner).toHaveAttribute('data-capability', 'e2ee', { timeout: 20_000 });

  // 3) Toggle encrypt → the worker encrypts the body client-side (real WASM).
  await dialog.locator('[data-testid="encrypt-toggle"]').check();
  await expect(dialog.locator('[data-testid="encrypted-draft-indicator"]')).toBeVisible({
    timeout: 20_000,
  });

  // 4) Send the armored ciphertext; it goes out over SMTP and loops back. (The
  // footer Send sits below the fold in the tall dialog and can't be scrolled into
  // the viewport; submit the form directly to fire the same onSubmit send path.)
  await dialog.locator('form.compose').evaluate((f) => (f as HTMLFormElement).requestSubmit());
  await expect(dialog).toBeHidden();

  // 5) The received message arrives in the inbox; open it.
  await waitForInboxMessage(page, subject);
  await openMessage(page, subject);

  // The reader recognizes the PGP armor and shows the unlock affordance.
  const decrypt = page.locator('[data-testid="reader-decrypt"]');
  await expect(decrypt).toBeVisible();

  // 6) Decrypt with the passphrase (real WASM, own key's encryptedPrivateBackup).
  await page.locator('[data-testid="decrypt-passphrase"]').fill(passphrase);
  await page.locator('[data-testid="decrypt-submit"]').click();

  // The plaintext renders in the sandboxed body iframe. The decrypted HTML was
  // sanitized in-worker: the markers survive, the <script>/handler/alert do NOT.
  const frame = page.frameLocator('iframe.reader__frame');
  await expect(frame.getByText(marker)).toBeVisible({ timeout: 20_000 });
  await expect(frame.getByText(boldMarker)).toBeVisible();

  // Assert the sanitization stripped the hostile bits from the rendered srcdoc.
  const srcdoc = await page.locator('iframe.reader__frame').getAttribute('srcdoc');
  expect(srcdoc, 'decrypted body srcdoc').not.toBeNull();
  expect(srcdoc!).not.toContain('<script');
  expect(srcdoc!).not.toContain('__pwned');
  expect(srcdoc!.toLowerCase()).not.toContain('onclick');

  // The window global the script would have set must NOT exist (no script ran —
  // and the frame is script-free anyway; belt-and-braces on the sanitize path).
  const leaked = await page.evaluate((t) => (window as Record<string, unknown>)[`__pwned_${t}`], token);
  expect(leaked).toBeUndefined();
});
