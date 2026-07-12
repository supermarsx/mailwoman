import { expect, type Page } from '@playwright/test';

/**
 * Credentials the mock JMAP backend accepts. The jmapUrl uses the compose
 * service name `mock` because mw-SERVER (not the browser) dials it; the browser
 * only ever talks to the same-origin proxy. Overridable for non-compose runs.
 */
export const CREDS = {
  jmapUrl: process.env['MW_E2E_JMAP_URL'] ?? 'http://mock:8181/.well-known/jmap',
  username: process.env['MW_E2E_USERNAME'] ?? 'testuser@example.org',
  password: process.env['MW_E2E_PASSWORD'] ?? 'testpass',
} as const;

/**
 * The Inbox mailbox button in the sidebar nav. Scoped to the "Mailboxes"
 * navigation so it matches neither the "Focused inbox" toggle (InboxTabs, which
 * substring-matches "inbox") NOR breaks when an unread badge makes the button's
 * accessible name "Inbox 3" instead of "Inbox".
 */
export function sidebarInbox(page: Page) {
  return page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' });
}

/**
 * Log in through the real UI and wait until the mailbox shell is ready.
 * Each spec calls this so tests stay independent (fresh session per test).
 */
export async function login(page: Page): Promise<void> {
  await page.goto('/');
  // The app boots (checks /api/me) before rendering the login form.
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

  await page.getByLabel('JMAP server URL').fill(CREDS.jmapUrl);
  await page.getByLabel('Username', { exact: true }).fill(CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();

  // Mailbox shell is up once the sidebar and Inbox mailbox render.
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await expect(sidebarInbox(page)).toBeVisible();
}

/** A message row in the list, located by (a substring of) its subject. */
export function messageRow(page: Page, subject: string) {
  return page.locator('.list__row').filter({ hasText: subject });
}

// ── Engine-mode helpers (shared by the V2 specs that need the real engine) ──
//
// The V2 modern-mail specs drive the SAME unmodified UI against mw-server in
// MW_MODE=engine over a real IMAP/SMTP account (Greenmail) on :8090 — the
// `engine` Playwright project. The server (not the browser) dials Greenmail, so
// the login "JMAP server URL" field carries the in-network `imap://greenmail:3143`
// and Greenmail's login name is the bare local part `testuser`.

export const ENGINE_CREDS = {
  imapUrl: process.env['MW_E2E_ENGINE_IMAP_URL'] ?? 'imap://greenmail:3143',
  username: process.env['MW_E2E_ENGINE_USERNAME'] ?? 'testuser',
  password: process.env['MW_E2E_ENGINE_PASSWORD'] ?? 'testpass',
  /** Full address used as the SMTP RCPT TO so a send loops back to this account. */
  selfAddress: process.env['MW_E2E_ENGINE_SELF'] ?? 'testuser@example.org',
} as const;

/** Log in through the real UI against the engine stack; waits for the shell. */
export async function engineLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
  await page.getByLabel('JMAP server URL').fill(ENGINE_CREDS.imapUrl);
  await page.getByLabel('Username', { exact: true }).fill(ENGINE_CREDS.username);
  await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
  await page.getByRole('button', { name: 'Sign in' }).click();
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await expect(sidebarInbox(page)).toBeVisible();
}

/** The whole list slot (row button + its action cluster), located by subject. */
export function messageSlot(page: Page, subject: string) {
  return page.locator('.list__slot').filter({ hasText: subject });
}

/**
 * Compose a self-addressed message and click Send. Does NOT wait for delivery —
 * the engine holds the submission for its undo-send window (~10 s) before it
 * dials SMTP, so callers use {@link waitForInboxMessage} to await arrival.
 * Optionally schedules the send for later (fills the datetime-local field).
 */
export async function composeSelf(
  page: Page,
  subject: string,
  body: string,
  opts: { sendLater?: string } = {},
): Promise<void> {
  await page.getByRole('button', { name: 'Compose' }).click();
  const dialog = page.getByRole('dialog', { name: 'Compose message' });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(body);
  if (opts.sendLater !== undefined) {
    await dialog.getByLabel('Send later', { exact: true }).fill(opts.sendLater);
    await expect(dialog.getByRole('button', { name: 'Schedule' })).toBeVisible();
    await dialog.getByRole('button', { name: 'Schedule' }).click();
  } else {
    await dialog.getByRole('button', { name: 'Send' }).click();
  }
  await expect(dialog).toBeHidden();
}

/**
 * Poll the Inbox (engine sync is poll-based) until a message with `subject`
 * shows up. Generous timeout: an undo-send hold (~10 s) plus SMTP loopback plus
 * the engine's watch-loop resync must all elapse first.
 */
export async function waitForInboxMessage(page: Page, subject: string, timeout = 45_000): Promise<void> {
  await expect(async () => {
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
  }).toPass({ timeout });
}
