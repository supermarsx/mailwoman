import { test, expect, type Page } from '@playwright/test';
import { engineLogin } from './helpers.ts';
import { uid } from './crypto-helpers.ts';

async function openRules(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Rules & filters' })).toBeVisible();
}

test('rule builder saves an authoritative script and reads it back after reload', async ({ page }) => {
  test.setTimeout(60_000);
  const token = uid();
  const ruleName = `Route ${token}`;
  const sender = `boss-${token}@example.com`;
  const mailbox = `Work-${token}`;
  await engineLogin(page);
  await openRules(page);
  const settings = page.getByRole('dialog', { name: 'Settings' });
  await settings.getByRole('button', { name: 'New rule', exact: true }).click();
  const builder = settings.getByRole('form', { name: 'Rule builder' });
  await builder.getByLabel('Rule name').fill(ruleName);
  await builder.getByLabel('Match value').first().fill(sender);
  await builder.getByLabel('Action value').first().fill(mailbox);
  await expect(builder.getByText('Engine', { exact: true }).first()).toBeVisible();
  await builder.getByRole('button', { name: 'Save rule' }).click();
  await expect(settings.getByText(ruleName)).toBeVisible();
  await settings.getByRole('tab', { name: 'Raw Sieve' }).click();
  const raw = settings.getByLabel('Raw Sieve source');
  await expect(raw).toHaveValue(new RegExp(ruleName));
  const source = await raw.inputValue();
  expect(source).toContain(`# rule: ${ruleName}`);
  expect(source).toContain('address :contains "from"');
  expect(source).toContain(sender);
  expect(source).toContain(`fileinto "${mailbox}"`);
  if (process.env['MW_SIEVE_SCREENSHOT_DIR']) {
    await raw.scrollIntoViewIfNeeded();
    await page.screenshot({ path: `${process.env['MW_SIEVE_SCREENSHOT_DIR']}/rules-raw.png` });
  }
  await expect(settings.getByRole('tab', { name: 'Dry run' })).toHaveCount(0);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  await openRules(page);
  await expect(page.getByRole('dialog', { name: 'Settings' }).getByText(ruleName)).toBeVisible();
});

test('vacation roundtrip and blocked external forwarding show real server policy', async ({ page }) => {
  test.skip(process.env['MW_SIEVE_BROWSER_POLICIES'] !== 'true', 'Requires fixture vacationAllowed=true and forward_external=false');
  test.setTimeout(60_000);
  await engineLogin(page);
  await openRules(page);
  const settings = page.getByRole('dialog', { name: 'Settings' });
  await settings.getByRole('tab', { name: 'Vacation reply', exact: true }).click();
  const panel = settings.locator('#rules-panel-vacation');
  await panel.getByLabel('Enable automatic replies').check();
  await panel.getByLabel('Reply text').fill('Away until tomorrow.');
  await panel.getByLabel('Subject', { exact: true }).fill('Out of office');
  await panel.getByLabel('Start (UTC)').fill('2026-01-01T00:00');
  await panel.getByLabel('End (UTC)').fill('2027-01-01T00:00');
  await panel.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(panel.getByText('Vacation reply saved.')).toBeVisible();
  await settings.getByRole('tab', { name: 'Raw Sieve' }).click();
  await expect(settings.getByLabel('Raw Sieve source')).toHaveValue(/vacation :days 7/);
  await settings.getByRole('tab', { name: 'Vacation reply', exact: true }).click();
  await expect(panel.getByLabel('Enable automatic replies')).toBeChecked();
  await expect(panel.getByLabel('Reply text')).toHaveValue('Away until tomorrow.');
  await settings.getByRole('tab', { name: 'Forwarding', exact: true }).click();
  const forwarding = settings.locator('#rules-panel-forward');
  await forwarding.getByLabel('Enabled', { exact: true }).check();
  await forwarding.getByLabel('Forward to', { exact: true }).fill('outside@elsewhere.test');
  await expect(forwarding.getByRole('alert')).toHaveText('Your administrator does not allow forwarding to this domain.');
  await expect(forwarding.getByRole('button', { name: 'Save', exact: true })).toBeDisabled();
  if (process.env['MW_SIEVE_SCREENSHOT_DIR']) {
    await forwarding.scrollIntoViewIfNeeded();
    await page.screenshot({ path: `${process.env['MW_SIEVE_SCREENSHOT_DIR']}/forward-policy.png` });
  }
});
