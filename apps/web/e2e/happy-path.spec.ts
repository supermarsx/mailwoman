import { test, expect } from '@playwright/test';
import { CREDS, login, messageRow } from './helpers.ts';

test.describe('happy path', () => {
  test('login -> read a message -> compose + send -> appears in Sent', async ({ page }) => {
    await login(page);

    // Mailbox sidebar shows both seeded mailboxes.
    await expect(page.getByRole('button', { name: 'Inbox' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sent' })).toBeVisible();

    // The seeded inbox messages are listed (proves the real Email/query+get proxy).
    await expect(messageRow(page, 'Welcome to Mailwoman')).toBeVisible();
    await expect(messageRow(page, 'Your invoice is ready')).toBeVisible();

    // Open a benign message; its sanitized body renders in the sandboxed iframe.
    await messageRow(page, 'Welcome to Mailwoman').click();
    await expect(page.getByRole('heading', { name: 'Welcome to Mailwoman' })).toBeVisible();
    const frame = page.locator('iframe[title="Message body"]');
    await expect(frame).toBeVisible();
    await expect
      .poll(async () => await frame.getAttribute('srcdoc'))
      .toContain('welcome aboard');

    // Compose a unique message to self and send it (real Email/set + EmailSubmission/set).
    const subject = `E2E ${Date.now()}`;
    await page.getByRole('button', { name: 'Compose' }).click();
    const dialog = page.getByRole('dialog', { name: 'Compose message' });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel('To', { exact: true }).fill(CREDS.username);
    await dialog.getByLabel('Subject', { exact: true }).fill(subject);
    await dialog.getByLabel('Body', { exact: true }).fill('Hello from the E2E happy path.');
    await dialog.getByRole('button', { name: 'Send' }).click();

    // Dialog closes on a successful send.
    await expect(dialog).toBeHidden();

    // Navigate to Sent and confirm the sent message is there after the re-query.
    await page.getByRole('button', { name: 'Sent' }).click();
    await expect(messageRow(page, subject)).toBeVisible();
  });
});
