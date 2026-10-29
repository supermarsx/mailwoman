// t39-e7 (L10 + L11) against a server built from the tree, no route mocks.
//
// MW_QUOTA_STACK says what the stack's IMAP account has:
//   quota  Dovecot with the quota plugin (scripts/dovecot-t32: 10 MiB "User quota")
//   none   a server without a quota (fresh GreenMail)
// MW_QUOTA_WARN=true when the admin set `quota.warnThresholds` low enough for the
// account's usage to pass the first one.
// MW_VACATION=hidden|engine: `sieve.vacationAllowed` off (default) or on, with no
// ManageSieve configured.
// MW_QUOTA_USER / MW_QUOTA_PASS sign in (default testuser / testpass).
// MW_QUOTA_SHOTS: a directory for screenshots.
import { test, expect } from '../csrf.ts';
import type { Page } from '@playwright/test';

const stack = process.env['MW_QUOTA_STACK'];
const shots = process.env['MW_QUOTA_SHOTS'];
const vacation = process.env['MW_VACATION'] ?? 'hidden';

test.skip(!stack, 'needs MW_QUOTA_STACK (see the header)');

async function signIn(page: Page, phone: boolean): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill(process.env['MW_QUOTA_USER'] ?? 'testuser');
  await page.getByLabel('Password', { exact: true }).fill(process.env['MW_QUOTA_PASS'] ?? 'testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible();
  if (phone) await page.getByRole('button', { name: 'Open folders and apps' }).click();
}

async function openSettings(page: Page, section: string): Promise<void> {
  await page.locator('.sidebar__footer').getByRole('button', { name: 'Settings', exact: true }).click();
  const nav = page.getByRole('navigation', { name: 'Settings sections' });
  await nav.getByRole('button', { name: section, exact: true }).click();
}

test('storage meter, Settings detail and vacation availability', async ({ page }, info) => {
  const phone = info.project.name === 'phone';
  await signIn(page, phone);
  const footer = page.locator('.sidebar__footer');
  if (stack === 'none') {
    // Give the meter's read time to land, then require that nothing appeared.
    await page.waitForTimeout(3000);
    await expect(footer.getByTestId('quota-meter')).toHaveCount(0);
  } else {
    const meter = footer.getByRole('meter', { name: 'Mailbox storage' });
    await expect(meter).toBeVisible();
    await expect(footer.getByTestId('quota-meter')).toContainText(/of 10 MB used/);
    if (process.env['MW_QUOTA_WARN'] === 'true') {
      await expect(footer.getByTestId('quota-meter')).toHaveAttribute('data-level', /warn|critical/);
      await expect(footer.getByRole('status')).toContainText(/Mailbox \d+% full/);
    } else {
      await expect(footer.getByTestId('quota-meter')).toHaveAttribute('data-level', 'ok');
    }
  }
  if (shots) await page.screenshot({ path: `${shots}/${stack}-${vacation}-${info.project.name}-sidebar.png` });

  await openSettings(page, 'Folders and sidebar');
  const details = page.getByTestId('quota-details');
  if (stack === 'none') {
    await page.waitForTimeout(1000);
    await expect(details).toHaveCount(0);
  } else {
    await expect(details.getByRole('rowheader', { name: 'Storage' })).toBeVisible();
    await expect(details).toContainText('Your mail server');
    await expect(details).toContainText(/of 10 MB \(\d+%\)/);
    await details.scrollIntoViewIfNeeded();
  }
  if (shots) await page.screenshot({ path: `${shots}/${stack}-${vacation}-${info.project.name}-settings-folders.png` });

  const nav = page.getByRole('navigation', { name: 'Settings sections' });
  await nav.getByRole('button', { name: 'Mail rules', exact: true }).click();
  const rules = page.getByRole('region', { name: 'Rules & filters' }).or(page.locator('[aria-labelledby="rules-heading"]'));
  await expect(rules.getByRole('tab', { name: 'Raw Sieve' })).toBeVisible();
  if (vacation === 'hidden') {
    await expect(rules.getByTestId('rules-vacation-unavailable')).toContainText('Vacation replies are not available for this account');
    await expect(rules.getByRole('tab', { name: 'Vacation reply', exact: true })).toHaveCount(0);
  } else {
    await expect(rules.getByTestId('rules-vacation-unavailable')).toHaveCount(0);
    await rules.getByRole('tab', { name: 'Vacation reply', exact: true }).click();
    const panel = rules.locator('#rules-panel-vacation');
    await expect(panel.getByTestId('vacation-runs-at')).toContainText('Mailwoman sends this reply when it syncs new inbox mail');
    await panel.getByLabel('Enable automatic replies').check();
    await panel.getByLabel('Reply text').fill(`Away (${info.project.name}).`);
    await panel.getByRole('button', { name: 'Save', exact: true }).click();
    await expect(panel.getByRole('status')).toHaveText('Vacation reply saved.');
    await panel.getByLabel('Enable automatic replies').uncheck();
    await panel.getByRole('button', { name: 'Save', exact: true }).click();
    await expect(panel.getByRole('status')).toHaveText('Saved. Automatic replies are off.');
  }
  if (shots) await page.screenshot({ path: `${shots}/${stack}-${vacation}-${info.project.name}-rules.png` });
});
