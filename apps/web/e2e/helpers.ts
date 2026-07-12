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
  await expect(page.getByRole('button', { name: 'Inbox' })).toBeVisible();
}

/** A message row in the list, located by (a substring of) its subject. */
export function messageRow(page: Page, subject: string) {
  return page.locator('.list__row').filter({ hasText: subject });
}
