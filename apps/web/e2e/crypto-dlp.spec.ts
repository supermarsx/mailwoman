import { test, expect } from '@playwright/test';
import { engineLogin, ENGINE_CREDS, uid } from './crypto-helpers.ts';
import { composeSurface } from './helpers.ts';

/**
 * V4 DLP live E2E (plan §3 e10 item 5 / SPEC §7.6): a compose containing a
 * Luhn-valid payment card number is BLOCKED on send by the engine-side DLP rule
 * (config-sourced via `MW_DLP_RULES`). The compose-time `Dlp/scan` dry-run surfaces
 * the block inline (the `dlpBlocked` gate), and the Send button is refused with the
 * rule message — the message never reaches the submission path.
 *
 * The engine must be started with a `MW_DLP_RULES` block rule on the built-in Luhn
 * PAN detector — the committed `docker-compose.crypto.yml` override the CI
 * `e2e-crypto` job layers on (rule name "Block card numbers"). No WASM worker is
 * involved — DLP is entirely server-side + the compose gate.
 */
test('DLP: a payment card number blocks the send with the rule message', async ({ page }) => {
  const token = uid();
  const subject = `DLP ${token}`;
  // A Luhn-valid test PAN (Visa test number). The engine's "pan" detector runs a
  // 13–19 digit + Luhn check, so this trips the block rule; the redacted audit row
  // never stores the number itself.
  const pan = '4111 1111 1111 1111';

  await engineLogin(page);

  await page.getByRole('button', { name: 'Compose', exact: true }).click();
  // Compose is a workspace tab region (af880dc1), not a modal dialog.
  const dialog = composeSurface(page);
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(`Please pay using card ${pan} today.`);

  // The crypto/DLP controls sit on the compose ribbon's Security tab (e0c99a79).
  await dialog.getByRole('tab', { name: 'Security', exact: true }).click();

  // The compose-time Dlp/scan dry-run surfaces the block inline.
  const block = dialog.locator('[data-testid="dlp-block"]');
  await expect(block).toBeVisible({ timeout: 15_000 });
  await expect(block).toHaveAttribute('data-action', 'block');
  // The rule name (frozen DlpVerdict.ruleName) is shown to the user — this matches
  // the committed docker-compose.crypto.yml block rule the CI job runs.
  await expect(block).toContainText('Block card numbers');

  // Send is gated: the compose refuses with the DLP message and stays open.
  await dialog.getByRole('button', { name: 'Send', exact: true }).click();
  await expect(dialog.locator('.login__error')).toContainText('data-loss-prevention rule');
  await expect(dialog).toBeVisible();
});
