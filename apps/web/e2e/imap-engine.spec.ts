import { test, expect, type Page } from '@playwright/test';
import { messageRow } from './helpers.ts';

/**
 * V1 engine-mode E2E: the SAME unmodified web UI, driven against mw-server in
 * MW_MODE=engine talking to a REAL IMAP/SMTP server (Greenmail) through
 * mw-engine — not the V0 JMAP mock. baseURL is :8090 (the `engine` project).
 *
 * The login form's "JMAP server URL" field is reinterpreted by engine mode as
 * an IMAP URL; the SERVER (not the browser) dials Greenmail over the compose
 * network, so the value is the in-network `imap://greenmail:3143`. Greenmail's
 * login name is the bare local part `testuser` (NOT the full address).
 *
 * This genuinely exercises the real seams:
 *   - IMAP LIST/SELECT -> the sidebar mailbox list (Inbox, role from SPECIAL-USE)
 *   - SMTP submission (mw-smtp -> Greenmail :3025) for the composed message
 *   - IMAP APPEND + SELECT/FETCH + MIME parse (mw-mime) for the message that
 *     comes back, sanitized and rendered in the sandboxed reader iframe.
 *
 * Greenmail's INBOX starts empty (fresh account), so there is nothing preseeded
 * to click. The spec therefore does the self-send-then-appears flow: compose a
 * uniquely-subjected message addressed to the account itself, send it, and
 * assert it turns up in the mailbox — proving send AND receive AND read.
 */

const ENGINE_CREDS = {
  // The server dials Greenmail; the browser only ever talks to :8090.
  imapUrl: process.env['MW_E2E_ENGINE_IMAP_URL'] ?? 'imap://greenmail:3143',
  username: process.env['MW_E2E_ENGINE_USERNAME'] ?? 'testuser',
  password: process.env['MW_E2E_ENGINE_PASSWORD'] ?? 'testpass',
  // Full address for the SMTP RCPT TO (delivers back to the same account).
  selfAddress: process.env['MW_E2E_ENGINE_SELF'] ?? 'testuser@example.org',
} as const;

/** Log in through the real UI against the engine stack. */
async function engineLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

  await page.getByLabel('JMAP server URL').fill(ENGINE_CREDS.imapUrl);
  await page.getByLabel('Username', { exact: true }).fill(ENGINE_CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();

  // Mailbox shell is up once the sidebar renders. The mailbox list comes from a
  // real IMAP LIST/SELECT, so Inbox (role=inbox via SPECIAL-USE) must appear.
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Inbox' })).toBeVisible();
}

test.describe('IMAP account through the unmodified web UI (engine mode)', () => {
  test('login -> real IMAP mailbox -> compose+send via SMTP -> arrives, MIME-parsed, in the sandboxed reader', async ({
    page,
  }) => {
    await engineLogin(page);

    // Sidebar mailbox list is the real IMAP folder list (Inbox at minimum).
    await expect(page.getByRole('button', { name: 'Inbox' })).toBeVisible();

    // Compose a NEW message addressed to the account itself. The subject is
    // unique so the assertion is immune to any leftover/seeded mail.
    const subject = `V1-E2E ${Date.now()}`;
    const bodyText = `engine-mode round trip ${subject}`;

    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
    await dialog.getByLabel('Subject', { exact: true }).fill(subject);
    await dialog.getByLabel('Body', { exact: true }).fill(bodyText);
    await dialog.getByRole('button', { name: 'Send' }).click();

    // Dialog closes only on a successful Email/set + EmailSubmission/set (the
    // send actually went out through mw-smtp to Greenmail).
    await expect(dialog).toBeHidden();

    // The self-addressed message is delivered by Greenmail and ingested by the
    // engine's sync. Sync is poll-based, so re-select Inbox until it appears.
    await expect(async () => {
      await page.getByRole('button', { name: 'Inbox' }).click();
      await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 30_000 });

    // Open it: the raw RFC822 was fetched over IMAP, parsed by mw-mime, and the
    // body sanitized. It renders in the locked-down (no allow-scripts /
    // allow-same-origin) reader iframe — the same reader component as V0.
    await messageRow(page, subject).click();
    await expect(page.getByRole('heading', { name: subject })).toBeVisible();

    const frame = page.locator('iframe[title="Message body"]');
    await expect(frame).toBeVisible();
    const sandbox = await frame.getAttribute('sandbox');
    expect(sandbox).not.toBeNull();
    expect(sandbox).not.toContain('allow-scripts');
    expect(sandbox).not.toContain('allow-same-origin');

    // The MIME-parsed, sanitized body content is present in the frame.
    await expect.poll(async () => await frame.getAttribute('srcdoc')).toContain(bodyText);
  });
});
