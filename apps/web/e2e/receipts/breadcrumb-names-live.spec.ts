// t95: breadcrumbs name every level, never a raw key or id, against a live
// server built from the tree (node scripts/e2e-receipts-stack.mjs).
// Admin console (at its configured path, admin-path.ts): a settings key page, a
// privacy policy key, a domain and its sub-page, a user, a role, an identity
// provider client and a security tab with its "new" page. User Settings: the
// category › section trail and an editor level.
// From the first navigation every text a breadcrumb label ever showed is
// recorded (a MutationObserver installed before the app starts), so a raw id
// that flashed for one frame before its name arrived fails the leg too.
// Chromium and Firefox at 1440 and 390. Shots go to MW_SCREENSHOT_DIR.
import { test, expect } from '../csrf.ts';
import { ADMIN } from '../admin/admin-path.ts';
import { mkdirSync } from 'node:fs';
import type { APIRequestContext, Browser, Page, TestInfo } from '@playwright/test';

const SHOTS = process.env['MW_SCREENSHOT_DIR'];
const SIZES = [{ name: '1440', width: 1440, height: 900 }, { name: '390', width: 390, height: 844 }] as const;
test.describe.configure({ timeout: 300000 });
const skipPhone = (info: TestInfo) => test.skip(info.project.name === 'phone', 'the 390 legs run in the chromium and firefox projects');
const shot = (info: TestInfo, name: string) => {
  if (!SHOTS) return info.outputPath(`${name}.png`);
  mkdirSync(SHOTS, { recursive: true });
  return `${SHOTS}/${info.project.name}-${name}.png`;
};

// Patterns of things a breadcrumb must never show.
const RAW: readonly [string, RegExp][] = [
  ['uuid', /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/i],
  ['hex id', /^[0-9a-f]{12,}$/i],
  ['message id', /^[a-z][a-z0-9]*(?:-[a-z0-9]+){1,}$/],
  ['marker', /^\+[a-z]+$/],
  ['dotted key', /^[a-z][a-zA-Z0-9]*(?:\.[a-zA-Z0-9*-]+)+$/],
];

/** Record every breadcrumb label text from the first frame on. */
async function recordCrumbs(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const seen = new Set<string>();
    (window as unknown as { __crumbs: Set<string> }).__crumbs = seen;
    const scan = () => { for (const el of document.querySelectorAll('[data-crumb-label]')) { const text = el.textContent?.trim(); if (text) seen.add(text); } };
    new MutationObserver(scan).observe(document, { subtree: true, childList: true, characterData: true });
  });
}
const seenCrumbs = (page: Page) => page.evaluate(() => [...(window as unknown as { __crumbs: Set<string> }).__crumbs]);

/** Wait until the trail has no loading level, then return its texts. */
async function trail(_page: Page, nav: ReturnType<Page["getByRole"]>): Promise<string[]> {
  await expect(nav).toBeVisible({ timeout: 30000 });
  await expect(nav.locator('[data-crumb-loading]')).toHaveCount(0, { timeout: 30000 });
  // textContent: the Settings trail is styled uppercase, the names are not.
  return (await nav.locator('li').allTextContents()).map((s) => s.trim());
}

function expectFriendly(texts: readonly string[], raws: readonly string[], allowed: readonly string[], where: string): void {
  for (const text of texts) {
    if (allowed.includes(text)) continue;
    expect(raws, `${where}: "${text}" is a raw route segment`).not.toContain(text);
    for (const [kind, re] of RAW) expect(re.test(text), `${where}: "${text}" looks like a ${kind}`).toBe(false);
  }
}

async function adminApi(playwright: { request: { newContext: (o: { baseURL: string }) => Promise<APIRequestContext> } }, info: TestInfo): Promise<APIRequestContext> {
  const api = await playwright.request.newContext({ baseURL: String(info.project.use.baseURL) });
  await new Promise((r) => setTimeout(r, 3100));
  expect((await api.post(`${ADMIN}/login`, { data: { username: 'root', password: 'hunter2' } })).status()).toBe(200);
  return api;
}

interface Fixture { domain: string; accountId: string; roleId: string; roleName: string; clientId: string; clientName: string }

/** The items the admin routes open: the domain, the signed-in user, a role, an IdP client. */
async function fixture(api: APIRequestContext, browser: Browser, info: TestInfo): Promise<Fixture> {
  // A user account exists once someone signed in.
  const user = await browser.newContext({ baseURL: String(info.project.use.baseURL), serviceWorkers: 'block', locale: 'en-US' });
  const p = await user.newPage();
  await signInUser(p);
  await user.close();
  const users = (await (await api.get(`${ADMIN}/users`)).json()) as { accountId: string; label?: string; address?: string }[];
  const account = users.find((u) => `${u.label ?? ''} ${u.address ?? ''}`.includes('testuser')) ?? users[0];
  expect(account, 'a user account').toBeTruthy();

  const domain = 't95-crumbs.example';
  const put = await api.put(`${ADMIN}/domains/${domain}`, { data: {} });
  expect([200, 201, 204, 409, 412, 428], `create domain: ${put.status()} ${await put.text()}`).toContain(put.status());
  expect((await api.get(`${ADMIN}/domains/${domain}`)).status()).toBe(200);

  const roles = ((await (await api.get(`${ADMIN}/roles`)).json()) as { roles: { id: string; name: string }[] }).roles;
  const role = roles[0]!;

  const clientName = 'T95 Breadcrumb Test Client With A Rather Long Display Name';
  const clients = ((await (await api.get(`${ADMIN}/oidc/clients`)).json()) as { clients: { clientId: string; name: string }[] }).clients;
  let client = clients.find((c) => c.name === clientName);
  if (!client) {
    // Client writes ask for a fresh password confirmation (step-up) first.
    expect((await api.post(`${ADMIN}/step-up`, { data: { password: 'hunter2' } })).status(), 'admin step-up').toBeLessThan(300);
    const made = await api.post(`${ADMIN}/oidc/clients`, { data: { name: clientName, config: { redirectUris: ['https://app.t95.example/callback'], postLogoutRedirectUris: [], scopes: ['openid'], authMethod: 'none' }, enabled: true, secretAction: 'keep' } });
    expect(made.status(), `create IdP client: ${await made.text()}`).toBeLessThan(300);
    client = { clientId: ((await made.json()) as { clientId: string }).clientId, name: clientName };
  }
  return { domain, accountId: account!.accountId, roleId: role.id, roleName: role.name, clientId: client.clientId, clientName };
}

async function signInUser(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByLabel('Email or username', { exact: true }).fill('testuser');
  await page.getByLabel('Password', { exact: true }).fill('testpass');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByLabel('Email or username', { exact: true })).toHaveCount(0, { timeout: 60000 });
  await expect(page.locator('.mail-pane')).toBeVisible({ timeout: 60000 });
}

async function signInAdmin(page: Page): Promise<void> {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.waitForTimeout(3100);
  const login = page.waitForResponse((r) => new URL(r.url()).pathname === `${ADMIN}/login` && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await login).status()).toBe(200);
}

for (const size of SIZES) {
  test(`admin console breadcrumbs name every level (${size.name})`, async ({ browser, playwright }, info) => {
    skipPhone(info);
    const api = await adminApi(playwright, info);
    const f = await fixture(api, browser, info);
    const context = await browser.newContext({ baseURL: String(info.project.use.baseURL), viewport: size, serviceWorkers: 'block', locale: 'en-US' });
    const page = await context.newPage();
    await recordCrumbs(page);
    await signInAdmin(page);
    const nav = page.getByRole('navigation', { name: 'Breadcrumb' });
    const ROUTES: { name: string; rest: string[]; expect: string[]; allowed?: string[] }[] = [
      { name: 'setting-key', rest: ['settings', 'mail.focusedInbox'], expect: ['Dashboard', 'Settings', 'Focused inbox'] },
      { name: 'setting-key-deep', rest: ['settings', 'security.signin.geoScope'], expect: ['Dashboard', 'Settings'] },
      { name: 'privacy-policy', rest: ['privacy', 'policy', 'privacy.trackers'], expect: ['Dashboard'] },
      { name: 'domain', rest: ['domains', f.domain], expect: ['Dashboard', 'Domains', f.domain], allowed: [f.domain] },
      { name: 'domain-connections', rest: ['domains', f.domain, 'connections'], expect: ['Dashboard', 'Domains', f.domain, 'Connections'], allowed: [f.domain] },
      { name: 'user', rest: ['users', f.accountId], expect: ['Dashboard', 'Users'] },
      { name: 'role', rest: ['roles', f.roleId, 'edit'], expect: ['Dashboard', 'Roles', f.roleName, 'Edit'], allowed: [f.roleName] },
      { name: 'idp-client', rest: ['identity-provider', 'clients', f.clientId, 'edit'], expect: ['Dashboard', 'Identity provider', 'Clients', f.clientName, 'Edit'] },
      { name: 'security-ban-new', rest: ['security', 'bans', 'new'], expect: ['Dashboard', 'Security', 'IP ban list', 'New'] },
    ];
    for (const r of ROUTES) {
      await page.goto(`${ADMIN}#/${r.rest.map(encodeURIComponent).join('/')}`);
      const texts = await trail(page, nav);
      console.log('T95_ADMIN', info.project.name, size.name, r.name, JSON.stringify(texts));
      expect(texts.length, r.name).toBe(r.rest.length + 1);
      for (const [i, want] of r.expect.entries()) {
        if (i === 1 && want !== texts[1]) expect(texts[1], `${r.name} section`).toContain(want.split(' ')[0]!);
        else expect(texts[i], `${r.name} level ${i}`).toBe(want);
      }
      expectFriendly(texts, r.rest.slice(1), r.allowed ?? [], r.name);
      // The current page is the last level.
      await expect(nav.locator('[aria-current="page"]')).toHaveCount(1);
      await expect(nav.locator('[aria-current="page"]')).toHaveText(texts.at(-1)!);
      await page.screenshot({ path: shot(info, `${size.name}-admin-${r.name}`) });
    }
    // The user's display name is an address or a name, not the account id.
    await page.goto(`${ADMIN}#/users/${encodeURIComponent(f.accountId)}`);
    const userTrail = await trail(page, nav);
    expect(userTrail.at(-1)).not.toBe(f.accountId);
    // A long name is cut with an ellipsis and offered whole as a tooltip.
    await page.goto(`${ADMIN}#/identity-provider/clients/${encodeURIComponent(f.clientId)}/edit`);
    await trail(page, nav);
    const client = nav.locator('[data-crumb-label]', { hasText: f.clientName });
    const cut = await client.evaluate((el) => el.scrollWidth > el.clientWidth + 1);
    if (cut) {
      await client.hover();
      // The name is an earlier level, so the link carries the tooltip.
      await expect(nav.locator('a', { hasText: f.clientName })).toHaveAttribute('data-tooltip', f.clientName);
      await expect(page.getByRole('tooltip')).toHaveText(f.clientName);
      await page.screenshot({ path: shot(info, `${size.name}-admin-long-tooltip`) });
    }
    console.log('T95_LONG', info.project.name, size.name, 'cut', cut);
    // Nothing raw was ever shown, not even for a frame.
    const seen = await seenCrumbs(page);
    console.log('T95_SEEN', info.project.name, size.name, JSON.stringify(seen));
    expectFriendly(seen, [f.accountId, f.clientId, f.roleId, 'mail.focusedInbox', 'security.signin.geoScope', 'privacy.trackers', 'new', 'edit', 'clients', 'connections', 'bans', 'policy'], [f.domain, f.roleName], 'ever shown');
    await context.close();
    await api.dispose();
  });

  test(`user Settings breadcrumbs name the category, section and editor (${size.name})`, async ({ browser }, info) => {
    skipPhone(info);
    const context = await browser.newContext({ baseURL: String(info.project.use.baseURL), viewport: size, serviceWorkers: 'block', locale: 'en-US' });
    const page = await context.newPage();
    await recordCrumbs(page);
    await signInUser(page);
    const sections = page.getByRole('navigation', { name: 'Settings sections' });
    if (!await sections.isVisible()) {
      let settings = page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first();
      if (!await settings.isVisible()) {
        const menu = page.locator('header.shell__bar button[aria-controls="shell-nav"]').first();
        if (await menu.isVisible() && (await menu.getAttribute('aria-expanded')) !== 'true') await menu.click();
        settings = page.locator('.sidebar, #shell-nav').getByRole('button', { name: 'Settings', exact: true }).first();
      }
      await settings.click();
    }
    await expect(sections).toBeVisible({ timeout: 30000 });
    const nav = page.getByRole('navigation', { name: 'Settings path' });
    const CASES: [string, string[]][] = [
      ['Appearance', ['Interface', 'Appearance']],
      ['Linked mailboxes', ['Account', 'Linked mailboxes']],
      ['Message display', ['Mail', 'Message display']],
      ['Labels', ['Mail', 'Labels']],
      ['Package tracking', ['Integrations', 'Package tracking']],
    ];
    for (const [button, want] of CASES) {
      const entry = sections.getByRole('button', { name: button, exact: true });
      if (!await entry.count()) { console.log('T95_SETTINGS_SKIP', button); continue; }
      await entry.click();
      const texts = await trail(page, nav);
      console.log('T95_SETTINGS', info.project.name, size.name, button, JSON.stringify(texts));
      expect(texts).toEqual(want);
      expectFriendly(texts, ['appearance', 'linked-accounts', 'reader', 'labels', 'packages', 'account', 'mail', 'interface', 'integrations'], [], button);
      await expect(nav.locator('[aria-current="page"]')).toHaveText(want.at(-1)!);
      await page.screenshot({ path: shot(info, `${size.name}-settings-${button.toLowerCase().replace(/\s+/g, '-')}`) });
    }
    // An editor adds a word level; the section becomes a way back.
    await sections.getByRole('button', { name: 'Labels', exact: true }).click();
    const create = page.getByRole('button', { name: /^New label/ }).first();
    if (await create.count()) {
      await create.click();
      const texts = await trail(page, nav);
      console.log('T95_SETTINGS_EDITOR', info.project.name, size.name, JSON.stringify(texts));
      expect(texts).toEqual(['Mail', 'Labels', 'New']);
      await page.screenshot({ path: shot(info, `${size.name}-settings-labels-new`) });
      await nav.getByRole('button', { name: 'Labels', exact: true }).click();
      expect(await trail(page, nav)).toEqual(['Mail', 'Labels']);
    } else console.log('T95_SETTINGS_EDITOR_SKIP no create-label button');
    const seen = await seenCrumbs(page);
    console.log('T95_SETTINGS_SEEN', info.project.name, size.name, JSON.stringify(seen));
    expectFriendly(seen, ['appearance', 'linked-accounts', 'reader', 'labels', 'packages'], [], 'ever shown');
    await context.close();
  });
}
