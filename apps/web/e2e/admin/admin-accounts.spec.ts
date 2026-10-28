import { test, expect, type Locator, type Page } from '@playwright/test';

// Real server and real SPA: no route mocks. The test creates only uniquely named
// accounts/roles and deletes them again through the same confirmation flow.
test('admin navigation, role creation, scoped assignments and deletion @phone', async ({ page }) => {
  const suffix = `${Date.now()}-${test.info().project.name}`;
  const username = `e2e-${suffix}`;
  const role = `Support ${suffix}`;
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill(process.env['MW_ADMIN_USER'] ?? 'root');
  await page.getByLabel('Password', { exact: true }).fill(process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  // Roles and admin accounts are edited on dedicated guided pages (2e8ad874); a write
  // may ask for the password again (step-up).
  await page.goto('/admin#/roles');
  await expect(page.getByRole('heading', { name: 'Roles', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'New role' })).toBeEnabled();
  await page.getByRole('button', { name: 'New role' }).click();
  await page.getByRole('textbox', { name: 'Display name' }).fill(role);
  await page.getByRole('button', { name: 'Continue', exact: true }).click();
  await page.locator('summary').filter({ hasText: 'Users and mail' }).click();
  await page.getByRole('checkbox', { name: 'Read users.read', exact: true }).check();
  await page.getByRole('button', { name: 'Continue', exact: true }).click();
  await page.getByRole('button', { name: 'Create role', exact: true }).click();
  await proveIfAsked(page, page.getByRole('button', { name: role, exact: true }));
  await expect(page.getByRole('button', { name: role, exact: true })).toBeVisible();
  await page.goto('/admin#/admins');
  await expect(page.getByRole('heading', { name: 'Admin accounts', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Create admin', exact: true }).click();
  await expect(page).toHaveURL(/#\/admins\/new$/);
  await page.getByRole('textbox', { name: 'Username', exact: true }).fill(username);
  await page.getByLabel('New password (at least 12 characters)').fill('e2e-password-long');
  await page.getByRole('button', { name: 'Create admin', exact: true }).click();
  await proveIfAsked(page, page.getByRole('checkbox', { name: role, exact: true }));
  await expect(page).toHaveURL(/#\/admins\/[^/]+\/edit$/);
  await page.getByRole('checkbox', { name: role, exact: true }).check();
  await page.getByRole('textbox', { name: `${role}: Domains, separated by commas (empty means all domains)`, exact: true }).fill('example.org');
  await page.getByRole('button', { name: 'Save role assignments' }).click();
  await proveIfAsked(page, page.getByText('Changes saved.', { exact: true }).last());
  await expect(page.getByText('Changes saved.', { exact: true }).last()).toBeVisible();
  await page.reload();
  await expect(page.getByRole('checkbox', { name: role, exact: true })).toBeChecked();
  await expect(page.getByRole('textbox', { name: `${role}: Domains, separated by commas (empty means all domains)`, exact: true })).toHaveValue('example.org');
  await page.screenshot({ path: test.info().outputPath('accounts.png'), fullPage: true });
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Confirm change' }).click();
  await proveIfAsked(page, page.getByRole('button', { name: 'Create admin', exact: true }));
  await expect(page).toHaveURL(/#\/admins$/);
  await expect(page.getByRole('button', { name: `${username} (${username})`, exact: true })).toHaveCount(0);
  await page.goto('/admin#/roles');
  await page.getByRole('button', { name: role, exact: true }).click();
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete', exact: true }).click();
  await proveIfAsked(page, page.getByRole('button', { name: 'New role', exact: true }));
  await expect(page.getByRole('button', { name: role, exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
});

/** Answer the step-up password prompt when the write asks for it; `next` is what the page shows otherwise. */
async function proveIfAsked(page: Page, next: Locator): Promise<void> {
  const proof = page.getByRole('dialog', { name: 'Confirm it is you' });
  await expect(proof.or(next).first()).toBeVisible();
  if (!(await proof.isVisible())) return;
  await proof.getByRole('textbox', { name: 'Password', exact: true }).fill(process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2');
  await proof.getByRole('button', { name: 'Confirm and continue' }).click();
  await expect(proof).toHaveCount(0);
}
