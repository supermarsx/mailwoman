import { test, expect } from '@playwright/test';
import {
  engineLogin,
  injectHtmlViaSmtp,
  uid,
  waitForInboxMessage,
  openMessage,
  cryptoAccountId,
  waitForMailRule,
} from './crypto-helpers.ts';

/**
 * V4 Security-panel + sender-controls live E2E (plan §3 e10 items 4 & 7 / SPEC
 * §7.3). The verdict is computed SERVER-SIDE by the engine (`SecurityVerdict/get`
 * via mail-auth + mail-parser) — no WASM worker — and rendered by the Reader's
 * SecurityPanel: a plain-language chip that expands into DKIM/SPF/DMARC/ARC verdicts,
 * the Received (delivery-path) chain, and the sender-controls block.
 *
 * Note on DKIM "pass": a cryptographically-verified DKIM PASS needs a valid
 * signature + a resolvable DNS key, which is deterministic only against a seeded
 * resolver (the Rust `mail-auth-verdicts` gate, e9) — not against a live Greenmail
 * loopback. This live E2E therefore asserts the panel RENDERS two DIFFERENT DKIM
 * verdicts (a signed message → a non-"none" verdict; an unsigned one → "not present")
 * plus the plain-language chip and the Received chain, per the item's literal ask.
 */

test('Security panel: renders DKIM verdicts + the Received chain for signed vs unsigned mail', async ({
  page,
}) => {
  const token = uid();
  const signedSubject = `DKIM signed ${token}`;
  const plainSubject = `DKIM none ${token}`;

  // A message carrying a DKIM-Signature (unverifiable here → a non-"none" verdict)
  // and one with none at all (→ "not present").
  await injectHtmlViaSmtp({
    from: 'Bank <notice@remote-example.net>',
    subject: signedSubject,
    text: 'signed body',
    html: '<p>signed body</p>',
    extraHeaders: [
      'DKIM-Signature: v=1; a=rsa-sha256; d=remote-example.net; s=sel; h=from:to:subject; bh=AAAA; b=BBBB',
    ],
  });
  await injectHtmlViaSmtp({
    from: 'Plain <plain@remote-example.net>',
    subject: plainSubject,
    text: 'plain body',
    html: '<p>plain body</p>',
  });

  await engineLogin(page);

  // ── Signed message: expand the panel, read the DKIM verdict + delivery path ──
  await waitForInboxMessage(page, signedSubject);
  await openMessage(page, signedSubject);
  const chip = page.locator('.reader button[aria-expanded]').first();
  await expect(chip).toBeVisible({ timeout: 15_000 });
  // The chip carries the plain-language summary.
  await expect(chip).not.toBeEmpty();
  await chip.click();
  const region = page.getByRole('region', { name: 'Message security details' });
  await expect(region).toBeVisible();
  await expect(region).toContainText('Authentication');
  await expect(region).toContainText('DKIM');
  // The Received chain (delivery path) renders at least one hop.
  await expect(region).toContainText('Delivery path');
  const signedDkim = await region.locator('text=/DKIM (passed|failed|neutral|not present|temporary error|permanent error)/').first().innerText();

  // ── Unsigned message: the DKIM verdict differs (not present) ──
  await openMessage(page, plainSubject);
  const chip2 = page.locator('.reader button[aria-expanded]').first();
  await expect(chip2).toBeVisible({ timeout: 15_000 });
  await chip2.click();
  const region2 = page.getByRole('region', { name: 'Message security details' });
  await expect(region2).toBeVisible();
  await expect(region2).toContainText('DKIM not present');
  const plainDkim = await region2.locator('text=/DKIM (passed|failed|neutral|not present|temporary error|permanent error)/').first().innerText();

  // The two messages produce genuinely different DKIM verdicts (signed ≠ unsigned).
  expect(signedDkim).not.toEqual(plainDkim);
});

test('Sender controls: "Block sender" creates a real MailRule/Sieve row (not localStorage)', async ({
  page,
}) => {
  const token = uid();
  const subject = `Block me ${token}`;
  const badSender = `block-${token}@spam.example.net`;

  await injectHtmlViaSmtp({
    from: `Spammer <${badSender}>`,
    subject,
    text: 'buy now',
    html: '<p>buy now</p>',
  });

  await engineLogin(page);
  await waitForInboxMessage(page, subject);
  await openMessage(page, subject);

  // Expand the security panel and block the sender through the real UI.
  const chip = page.locator('.reader button[aria-expanded]').first();
  await expect(chip).toBeVisible({ timeout: 15_000 });
  await chip.click();
  const region = page.getByRole('region', { name: 'Message security details' });
  await expect(region).toBeVisible();
  await region.getByRole('button', { name: 'Block sender', exact: true }).click();
  // The panel confirms the action (SenderControl/set resolved).
  await expect(region.getByRole('status')).not.toBeEmpty({ timeout: 10_000 });

  // Assert the REAL mechanism: a MailRule referencing the blocked sender exists on
  // the engine (block → Sieve From-is → Move Junk + Stop), queried over the same
  // cookie-authed session — not a localStorage flag (plan §1.9 / risk #14).
  const acct = await cryptoAccountId(page.request);
  const rule = await waitForMailRule(page.request, acct, badSender);
  expect(JSON.stringify(rule)).toContain(badSender);
});
