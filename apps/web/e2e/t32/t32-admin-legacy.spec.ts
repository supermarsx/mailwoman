import AxeBuilder from '@axe-core/playwright';
import { test, expect, type Page } from '@playwright/test';

/**
 * The admin screens f33 rebuilt on the UX templates (t32-cfg §4.12, t32-admin §3.1 e16),
 * each through its primary task in a real browser against a real server:
 *
 *  - Security policy: change the reading floor, save, reload, it is kept (then put back);
 *  - OAuth clients: list a redirect host and turn self-registration on, save, reload, kept
 *    (then put back);
 *  - Single sign-on: add an OIDC provider in the form at #/sso/new, it is listed and opens
 *    at #/sso/<id>/edit; deleting it asks first; SAML metadata uploaded as a file is previewed and
 *    fills the sign-in address;
 *  - Plugins: register a first-party component and see the server's word on why it is not
 *    running (or its refusal at the field), uninstall asks for the id;
 *  - UI plugins: register an unsigned manifest read from pasted text, then delete it;
 *  - AI assistant: grant a capability, save, reload, kept (then put back);
 *  - Server metadata: the account picker or the empty state.
 *
 * Every section is axe-clean (WCAG 2.2 AA, colour contrast included) at 1440x900, and at
 * 390x844 (@phone) it has no horizontal page scroll.
 *
 * Runs against an engine-mode server whose admin console accepts MW_ADMIN_USER /
 * MW_ADMIN_PASSWORD (default root / hunter2, the dev and CI stacks' values; DEV/CI ONLY).
 * On a stack that refuses them the tests skip and say so.
 */

const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? 'root';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';
const ADMIN_PATH = process.env['MW_E2E_ADMIN_PATH'] ?? '/admin';
const WCAG_22_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const DESKTOP = { width: 1440, height: 900 };
const PHONE = { width: 390, height: 844 };

const SECTIONS = [
  ['security-policy', 'Security policy'],
  ['oauth-clients', 'OAuth clients'],
  ['sso', 'Single sign-on'],
  ['plugins', 'Plugins'],
  ['ui-plugins', 'UI plugins'],
  ['ai-assistant', 'AI assistant'],
  ['server-metadata', 'Server metadata'],
] as const;

async function signIn(page: Page): Promise<void> {
  const res = await page.request.post(`${ADMIN_PATH}/login`, { data: { username: ADMIN_USER, password: ADMIN_PASSWORD } });
  test.skip(res.status() === 401, 'the admin console refuses MW_ADMIN_USER/MW_ADMIN_PASSWORD on this stack');
  expect(res.status(), 'admin login').toBe(200);
}

async function open(page: Page, section: string, heading: string): Promise<void> {
  await page.goto(`${ADMIN_PATH}#/${section}`);
  await expect(page.getByRole('heading', { level: 2, name: heading })).toBeVisible();
}

async function axe(page: Page, label: string): Promise<void> {
  const v = (await new AxeBuilder({ page }).withTags(WCAG_22_AA).include('#admin-main').analyze()).violations;
  const lines = v.map((r) => `  [${r.impact ?? 'n/a'}] ${r.id}: ${r.nodes.slice(0, 4).map((n) => n.target.join(' ')).join(' | ')}`);
  if (v.length > 0) console.error(`axe [${label}] ${v.length} violation(s):\n${lines.join('\n')}`);
  expect(lines, `axe violations on ${label}`).toEqual([]);
}

async function noHorizontalScroll(page: Page, label: string): Promise<void> {
  const { scroll, width } = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, width: window.innerWidth }));
  expect(scroll, `horizontal page scroll on ${label}`).toBeLessThanOrEqual(width);
}

/** Save the settings form on screen and wait for its result. */
async function save(page: Page, name = 'Save'): Promise<void> {
  await page.getByRole('button', { name, exact: true }).click();
  await expect(page.getByTestId('save-result')).toHaveText(/Saved/);
}

test.beforeEach(async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.setViewportSize(DESKTOP);
  await signIn(page);
});

test.describe('rebuilt admin sections', () => {
  test('Security policy: the reading floor is saved and kept across a reload', async ({ page }) => {
    await open(page, 'security-policy', 'Security policy');
    const floor = page.getByRole('combobox', { name: 'Reading floor' });
    const before = await floor.inputValue();
    const next = before === 'plain-text' ? 'sanitized-no-media' : 'plain-text';
    await floor.selectOption(next);
    await expect(page.getByTestId('save-bar')).toContainText('1 unsaved change');
    await save(page);
    await page.reload();
    await expect(page.getByRole('combobox', { name: 'Reading floor' })).toHaveValue(next);
    await axe(page, 'security-policy');
    await page.getByRole('combobox', { name: 'Reading floor' }).selectOption(before);
    await save(page);
  });

  test('OAuth clients: a redirect host and self-registration are saved and kept', async ({ page }) => {
    await open(page, 'oauth-clients', 'OAuth clients');
    const hosts = page.getByRole('textbox', { name: 'Allowed redirect hosts' });
    await hosts.fill('apps.example.org');
    await hosts.press('Enter');
    await page.getByRole('checkbox', { name: 'Let apps register themselves' }).check();
    await save(page);
    await page.reload();
    await expect(page.getByRole('checkbox', { name: 'Let apps register themselves' })).toBeChecked();
    await expect(page.getByRole('list', { name: 'Allowed redirect hosts' })).toContainText('apps.example.org');
    await axe(page, 'oauth-clients');
    await page.getByRole('checkbox', { name: 'Let apps register themselves' }).uncheck();
    await page.getByRole('button', { name: /Remove .*apps\.example\.org/ }).click();
    await save(page);
  });

  test('Single sign-on: add an OIDC provider, open it, delete it after confirming', async ({ page }) => {
    // Since 041b9bdb the editor is its own page (#/sso/new, #/sso/<id>/edit) and a
    // save returns to the provider table; the table no longer shares the screen.
    const id = `f33-e2e-${Date.now().toString(36)}`;
    await open(page, 'sso', 'Single sign-on');
    await page.getByRole('button', { name: 'Add a login backend' }).first().click();
    await expect(page).toHaveURL(/#\/sso\/new$/);
    const detail = page.getByTestId('sso-editor-page');
    await detail.getByRole('textbox', { name: 'Backend ID' }).fill(id);
    await detail.getByRole('textbox', { name: 'Display name' }).fill('E2E provider');
    await detail.getByRole('textbox', { name: 'Issuer URL' }).fill('https://idp.example.org/realms/e2e');
    await detail.getByRole('textbox', { name: 'Client ID' }).fill('mailwoman');
    await detail.getByRole('button', { name: 'Add backend' }).click();
    await expect(page).toHaveURL(/#\/sso$/);
    const row = page.locator(`[data-row-key="${id}"]`);
    await expect(row).toContainText('E2E provider');
    await axe(page, 'sso list');
    await row.getByRole('button', { name: 'Edit E2E provider', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`#/sso/${id}/edit$`));
    await expect(detail.getByRole('heading', { name: 'E2E provider' })).toBeVisible();
    await expect(detail.getByRole('textbox', { name: 'Backend ID' })).toHaveAttribute('readonly', '');
    await axe(page, 'sso detail');
    await detail.getByRole('button', { name: 'Back to sign-in providers' }).click();
    await expect(page).toHaveURL(/#\/sso$/);
    await row.getByRole('button', { name: /^Delete/ }).click();
    const dialog = page.getByRole('alertdialog');
    await expect(dialog).toContainText('Delete');
    await dialog.getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(row).toHaveCount(0);
  });

  test('Single sign-on: SAML metadata from a file is previewed and fills the sign-in address', async ({ page }) => {
    const xml = `<?xml version="1.0"?><md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="https://idp.example.org/saml"><md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol"><md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="https://idp.example.org/saml/sso"/></md:IDPSSODescriptor></md:EntityDescriptor>`;
    await page.goto(`${ADMIN_PATH}#/sso/new`);
    const detail = page.getByTestId('sso-editor-page');
    await detail.getByRole('combobox', { name: 'Protocol' }).selectOption('saml');
    await detail.getByLabel('Metadata file').setInputFiles({ name: 'idp.xml', mimeType: 'application/xml', buffer: Buffer.from(xml) });
    const preview = detail.getByTestId('sso-metadata-preview');
    await expect(preview).toContainText('https://idp.example.org/saml/sso');
    await preview.getByRole('button', { name: 'Use these values' }).click();
    await expect(detail.getByRole('textbox', { name: 'IdP SSO URL' })).toHaveValue('https://idp.example.org/saml/sso');
    await axe(page, 'sso saml form');
  });

  test('Plugins: registering a first-party component shows what the server says, uninstall asks for the id', async ({ page }) => {
    await open(page, 'plugins', 'Plugins');
    const existing = page.locator('[data-row-key="spam-rspamd"]');
    if ((await existing.count()) === 0) {
      await page.getByRole('button', { name: 'Register a plugin' }).first().click();
      const form = page.getByTestId('plugin-register');
      await form.getByRole('combobox', { name: 'Component' }).selectOption('spam-rspamd');
      await form.getByRole('button', { name: 'Register' }).click();
      // Either the plugin opens with the server's status line, or the refusal is shown at the field.
      await expect(page.getByTestId('plugin-card').or(form.locator('[aria-invalid="true"]'))).toBeVisible();
      if ((await page.getByTestId('plugin-card').count()) === 0) {
        await axe(page, 'plugins register refusal');
        return;
      }
    } else {
      await page.goto(`${ADMIN_PATH}#/plugins/spam-rspamd`);
    }
    const card = page.getByTestId('plugin-card');
    await expect(card.getByTestId('status-line')).toContainText('Not loaded');
    await axe(page, 'plugins detail');
    await card.getByRole('button', { name: /^Uninstall/ }).click();
    const dialog = page.getByRole('alertdialog');
    await expect(dialog.getByRole('button', { name: 'Uninstall', exact: true })).toBeDisabled();
    await dialog.getByRole('textbox').fill('spam-rspamd');
    await dialog.getByRole('button', { name: 'Uninstall', exact: true }).click();
    await expect(page.locator('[data-row-key="spam-rspamd"]')).toHaveCount(0);
  });

  test('UI plugins: an unsigned manifest is previewed, registered when allowed, and deleted after confirming', async ({ page }) => {
    const id = `f33-e2e-${Date.now().toString(36)}`;
    await page.goto(`${ADMIN_PATH}#/ui-plugins/%2Bregister`);
    const form = page.getByTestId('ui-plugin-register');
    await form.getByRole('textbox', { name: 'Or paste the manifest' }).fill(JSON.stringify({ id, name: 'E2E toolbar', version: '1.0.0', capabilities: ['ui:message-toolbar'], extensionPoints: ['message-toolbar'] }));
    await expect(form.getByTestId('ui-manifest-preview')).toContainText(id);
    await form.getByRole('checkbox', { name: 'Allow this plugin to register without a signature' }).check();
    await form.getByRole('button', { name: 'Register' }).click();
    const card = page.getByTestId('ui-plugin-card');
    await expect(card).toHaveAttribute('data-plugin-id', id);
    await axe(page, 'ui-plugins detail');
    await card.getByRole('button', { name: /^Delete/ }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(page.locator(`[data-row-key="${id}"]`)).toHaveCount(0);
  });

  test('AI assistant: a capability grant is saved and kept across a reload', async ({ page }) => {
    await open(page, 'ai-assistant', 'AI assistant');
    const recap = page.getByRole('checkbox', { name: 'Recap' });
    const was = await recap.isChecked();
    await recap.setChecked(!was);
    await save(page, 'Save configuration');
    await page.reload();
    await expect(page.getByRole('checkbox', { name: 'Recap' })).toBeChecked({ checked: !was });
    await axe(page, 'ai-assistant');
    await page.getByRole('checkbox', { name: 'Recap' }).setChecked(was);
    await save(page, 'Save configuration');
  });

  test('Server metadata: the account picker, or the empty state with its next step', async ({ page }) => {
    await open(page, 'server-metadata', 'Server metadata');
    await expect(page.getByRole('combobox', { name: 'Account' }).or(page.getByRole('button', { name: 'Add a user' }))).toBeVisible();
    await axe(page, 'server-metadata');
  });
});

test.describe('rebuilt admin sections on a phone', () => {
  for (const [section, heading] of SECTIONS) {
    test(`${heading} has no horizontal page scroll and no axe violations at 390x844 @phone`, async ({ page }) => {
      await page.setViewportSize(PHONE);
      await open(page, section, heading);
      // The section has read its data (no loading line left in the main column).
      await expect(page.locator('#admin-main').getByText('Loading…')).toHaveCount(0);
      await noHorizontalScroll(page, `${section} (phone)`);
      await axe(page, `${section} (phone)`);
    });
  }
});
