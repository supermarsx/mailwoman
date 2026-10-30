import { expect, type APIRequest, type APIRequestContext, type APIResponse, type Locator, type Page } from '@playwright/test';
import net from 'node:net';

/**
 * Credentials the mock JMAP backend accepts. Which JMAP server they sign in to
 * is the stack's server-side `MW_UPSTREAM_DEFAULT` (the compose service `mock`):
 * a sign-in never names a server (t32-h9). Overridable for non-compose runs.
 */
export const CREDS = {
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
 * Open the app and sign in through the login screen, which asks for the email
 * address (or user name) and the password only. The server picks the mail server
 * from its own configuration (`MW_UPSTREAM_DEFAULT` in every test stack, t32-h9).
 * `serverUrl` is accepted and ignored so older call sites keep compiling; there
 * is no field to type it into. Returns once the form has been submitted; the
 * caller waits for whatever its viewport shows next.
 */
export async function signInManually(
  page: Page,
  creds: { serverUrl?: string; username: string; password: string },
): Promise<void> {
  await page.goto('/');
  // The app boots (checks /api/me) before rendering the login form.
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
  await page.getByLabel('Email or username', { exact: true }).fill(creds.username);
  await page.getByLabel('Password', { exact: true }).fill(creds.password);
  // mw-server paces sign-ins per client IP (`signin.retryCooldownSecs`, default 3 s,
  // d94472d7). Every spec signs in from the same address, so a test that starts
  // within that window of the previous one is answered 429 `signinCooldown` with a
  // Retry-After. That is the product working as specified, not a failure of the
  // spec: wait out the window it names and submit the same form again. Stacks that
  // set the policy to 0 (see `disableSigninCooldown`) never take this branch.
  for (let attempt = 0; ; attempt += 1) {
    const answer = page.waitForResponse(
      (r) => r.request().method() === 'POST' && new URL(r.url()).pathname.endsWith('/api/login'),
    );
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    const res = await answer;
    if (res.status() !== 429 || attempt >= 3) return;
    const body = (await res.json().catch(() => null)) as { code?: unknown } | null;
    if (body?.code !== 'signinCooldown') return;
    const wait = Number(res.headers()['retry-after'] ?? '3');
    await page.waitForTimeout((Number.isFinite(wait) ? Math.min(wait, 30) : 3) * 1000 + 250);
  }
}

/**
 * A narrow ribbon folds whole groups into "More <group> commands" buttons. Make
 * `control` visible: as it is, or by opening each folded group in `scope` in turn
 * (closing the previous one with Escape) until it shows. Fails when no group holds it.
 */
export async function revealRibbonControl(page: Page, scope: Locator, control: Locator): Promise<Locator> {
  // A folded group keeps its controls laid out off screen for measuring, which
  // Playwright still counts as visible: shown means a box inside the viewport.
  const shown = async (): Promise<boolean> => {
    if (!(await control.isVisible().catch(() => false))) return false;
    const box = await control.boundingBox().catch(() => null);
    const size = page.viewportSize();
    return !!box && !!size && box.width > 1 && box.x >= 0 && box.y >= 0 && box.x + box.width <= size.width + 1 && box.y + box.height <= size.height + 1;
  };
  // A newly mounted/tab-switched ribbon folds over successive animation frames.
  // Retry discovery as well as the visibility assertion, so an early empty set
  // of More buttons cannot strand the test waiting for a folded-away control.
  await expect(async () => {
    if (await shown()) return;
    const folded = scope.getByRole('button', { name: /^More .+ commands$/ }).filter({ visible: true });
    const count = await folded.count();
    for (let i = 0; i < count; i += 1) {
      const more = folded.nth(i);
      await more.click({ timeout: 1000 });
      for (let tries = 0; tries < 5 && !(await shown()); tries += 1) await page.waitForTimeout(100);
      if (await shown()) return;
      await page.keyboard.press('Escape');
    }
    expect(await shown(), 'the control is on screen, inline or in a folded group').toBe(true);
  }).toPass({ timeout: 10000 });
  return control;
}

/** Isolate full-width editor/geometry tests from a previous saved split layout. */
export async function useTabbedWorkspace(page: Page): Promise<() => Promise<void>> {
  const endpoint = '/api/account/preferences';
  const original = await (await page.request.get(endpoint)).json() as { preferences: Record<string, unknown> };
  const authority = (await (await page.request.get('/api/me')).json() as { storageAuthority: string }).storageAuthority;
  await page.getByRole('group', { name: 'View arrangement' }).getByRole('button', { name: 'Tabs', exact: true }).click();
  await expect.poll(async () => (await (await page.request.get(endpoint)).json()).preferences.workspaceLayout).toBe('tabs');
  return async () => {
    const current = await page.request.get(endpoint);
    const document = await current.json() as { version: number; preferences: Record<string, unknown> };
    if (original.preferences['workspaceLayout'] === undefined) delete document.preferences['workspaceLayout'];
    else document.preferences['workspaceLayout'] = original.preferences['workspaceLayout'];
    expect((await page.request.put(endpoint, {
      headers: { 'if-match': current.headers()['etag']!, 'x-mailwoman-storage-authority': authority }, data: document,
    })).status()).toBe(200);
  };
}

/**
 * Set one of testuser's own settings for the length of a test, from a request
 * context of its own (so it works before the spec signs in and after it signs
 * out), and return the undo: the earlier user value back, or the user value
 * removed when there was none. `request` is `playwright.request` from the test
 * fixture, so writes carry the CSRF header (csrf.ts); every write also carries
 * the session's storage authority, as the web client sends it.
 */
export async function withUserSetting(
  request: APIRequest,
  baseURL: string,
  key: string,
  value: unknown,
): Promise<() => Promise<void>> {
  const session = async <T>(work: (ctx: APIRequestContext, headers: Record<string, string>) => Promise<T>): Promise<T> => {
    const ctx = await request.newContext({ baseURL });
    try {
      const login = await postSignIn(ctx, '/api/login', { username: 'testuser', password: 'testpass' });
      expect(login.ok(), `sign in for ${key}: ${login.status()}`).toBe(true);
      const me = (await (await ctx.get('/api/me')).json()) as { storageAuthority: string };
      return await work(ctx, { 'x-mailwoman-storage-authority': me.storageAuthority });
    } finally {
      await ctx.dispose();
    }
  };
  const before = await session(async (ctx, headers) => {
    const settings = (await (await ctx.get('/api/settings')).json()) as { settings: Record<string, { source: string; value: unknown } | undefined> };
    const put = await ctx.put(`/api/settings/${key}`, { headers, data: { value } });
    expect(put.ok(), `PUT ${key}: ${put.status()}`).toBe(true);
    return settings.settings[key];
  });
  return async () => {
    await session(async (ctx, headers) => {
      const undo = before?.source === 'user'
        ? await ctx.put(`/api/settings/${key}`, { headers, data: { value: before.value } })
        : await ctx.delete(`/api/settings/${key}`, { headers });
      expect(undo.ok(), `restore ${key}: ${undo.status()}`).toBe(true);
    });
  };
}

/**
 * Below 600 px the composer ribbon starts in auto-hide (9dab0fe7, D4), with its
 * groups folded until a tab is pressed; an explicit "pinned" keeps it open
 * (composer-tools-live checks both). Specs about what the ribbon's controls DO
 * pin it on the phone project as a user would, through `ui.bar.composerRibbon`,
 * and put the setting back afterwards. Call from `test.beforeEach`; it returns
 * the undo for `test.afterEach` (a no-op on the desktop projects).
 */
export async function pinComposerRibbonOnPhone(
  request: APIRequest,
  baseURL: string | undefined,
  project: string,
): Promise<() => Promise<void>> {
  if (project !== 'phone') return async () => {};
  return withUserSetting(request, baseURL ?? '', 'ui.bar.composerRibbon', 'pinned');
}

/**
 * POST a JMAP request the way the web client does (`src/api/client.ts`): the
 * server admits a few JMAP requests at once (`maxConcurrentRequests`) and answers
 * the rest 429 `urn:ietf:params:jmap:error:limit`. A spec that calls `/jmap/api`
 * while the app it just signed in to is still loading is one more client, so it
 * waits and resends on that answer (100, 250, 500, 1000, 2000 ms, then 2 s for up
 * to 20 s in all). Any other answer, including any other 429, is returned as is.
 */
export async function postJmap(
  ctx: APIRequestContext,
  options: { data: unknown; headers?: Record<string, string> },
): Promise<APIResponse> {
  const waits = [100, 250, 500, 1000, 2000, ...Array<number>(8).fill(2000)];
  let res = await ctx.post('/jmap/api', options);
  for (const wait of waits) {
    if (res.status() !== 429) break;
    const body = (await res.json().catch(() => null)) as { type?: unknown; limit?: unknown } | null;
    if (body?.type !== 'urn:ietf:params:jmap:error:limit' || body.limit !== 'maxConcurrentRequests') break;
    await new Promise((resolve) => setTimeout(resolve, wait));
    res = await ctx.post('/jmap/api', options);
  }
  return res;
}

/**
 * POST a sign-in over the API, waiting out the server's sign-in cooldown (429
 * `signinCooldown` + Retry-After, see `signInManually`) up to three times.
 */
export async function postSignIn(
  ctx: APIRequestContext,
  url: string,
  data: Record<string, unknown>,
): Promise<APIResponse> {
  let res = await ctx.post(url, { data });
  for (let attempt = 0; res.status() === 429 && attempt < 3; attempt += 1) {
    const body = (await res.json().catch(() => null)) as { code?: unknown } | null;
    if (body?.code !== 'signinCooldown') break;
    const wait = Number(res.headers()['retry-after'] ?? '3');
    await new Promise((resolve) => setTimeout(resolve, (Number.isFinite(wait) ? Math.min(wait, 30) : 3) * 1000 + 250));
    res = await ctx.post(url, { data });
  }
  return res;
}

/**
 * Admin console credentials of the engine stack (`MW_ADMIN_USER` /
 * `MW_ADMIN_PASSWORD` of `mailwoman-engine` in docker-compose.dev.yml).
 */
export const ENGINE_ADMIN = {
  username: process.env['MW_E2E_ADMIN_USER'] ?? 'root',
  password: process.env['MW_E2E_ADMIN_PASSWORD'] ?? 'hunter2',
} as const;

/**
 * Set the stack's sign-in pacing policy (`signin.retryCooldownSecs`) to 0 through
 * the admin settings API, so specs that sign in once per test do not each wait out
 * the per-IP window (see `signInManually`). Test stacks only: it changes a global
 * policy row. Idempotent (skips the write when the value is already 0) and uses a
 * request context of its own, so no admin cookie reaches the page under test.
 *
 * `factory` is the worker's `playwright.request`; `baseURL` is the stack's origin
 * (a project's `use.baseURL`).
 */
export async function disableSigninCooldown(factory: APIRequest, baseURL: string): Promise<void> {
  const ctx = await factory.newContext({ baseURL });
  try {
    const signedIn = await postSignIn(ctx, '/admin/login', ENGINE_ADMIN);
    expect(signedIn.status(), 'admin API sign-in for the cooldown policy').toBe(200);
    const url = '/admin/settings/signin.retryCooldownSecs';
    const current = await ctx.get(url);
    expect(current.status(), `GET ${url}`).toBe(200);
    const policy = (await current.json()) as { stored?: { globalValue?: unknown } | null };
    if (policy.stored?.globalValue === 0) return;
    const { cookies } = await ctx.storageState();
    const csrf = cookies.find((c) => c.name === 'mw_csrf')?.value ?? '';
    const put = await ctx.put(url, {
      headers: { 'if-match': current.headers()['etag'] ?? '', 'x-csrf-token': csrf },
      data: { globalValue: 0 },
    });
    expect(put.ok(), `PUT ${url} {globalValue:0} ⇒ ${put.status()} ${await put.text()}`).toBe(true);
  } finally {
    await ctx.dispose();
  }
}

/**
 * Log in through the real UI and wait until the mailbox shell is ready.
 * Each spec calls this so tests stay independent (fresh session per test).
 */
export async function login(page: Page): Promise<void> {
  await signInManually(page, { username: CREDS.username, password: CREDS.password });

  // Mailbox shell is up once the sidebar and Inbox mailbox render.
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
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
// `engine` Playwright project. The server (not the browser) dials Greenmail at
// the stack's `MW_UPSTREAM_DEFAULT` (`imap://greenmail:3143`); the login screen
// takes Greenmail's login name, the bare local part `testuser`.

export const ENGINE_CREDS = {
  /** The IMAP server the engine stacks set as `MW_UPSTREAM_DEFAULT`; never sent by a sign-in. */
  imapUrl: process.env['MW_E2E_ENGINE_IMAP_URL'] ?? 'imap://greenmail:3143',
  username: process.env['MW_E2E_ENGINE_USERNAME'] ?? 'testuser',
  password: process.env['MW_E2E_ENGINE_PASSWORD'] ?? 'testpass',
  /** Full address used as the SMTP RCPT TO so a send loops back to this account. */
  selfAddress: process.env['MW_E2E_ENGINE_SELF'] ?? 'testuser@example.org',
} as const;

/** Log in through the real UI against the engine stack; waits for the shell. */
export async function engineLogin(page: Page): Promise<void> {
  await signInManually(page, {
    username: ENGINE_CREDS.username,
    password: ENGINE_CREDS.password,
  });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  await expect(sidebarInbox(page)).toBeVisible();
}

// ── Phone-width variants ────────────────────────────────────────────────────
//
// Below 761 px the sidebar is a closed drawer, so `engineLogin` and
// `waitForInboxMessage` — which wait on and click the sidebar's Inbox button —
// do not work there. These are the same two steps through the drawer.

/** The top-bar button that opens the sidebar drawer at phone width. */
export function drawerButton(page: Page) {
  return page.getByRole('button', { name: 'Open folders and apps' });
}

/** Sign in to the engine stack at phone width; ready once the top bar is up. */
export async function enginePhoneLogin(page: Page): Promise<void> {
  await signInManually(page, {
    username: ENGINE_CREDS.username,
    password: ENGINE_CREDS.password,
  });
  await expect(drawerButton(page)).toBeVisible();
}

/**
 * {@link waitForInboxMessage} at phone width: re-select the Inbox through the
 * drawer (which re-queries the engine) until `subject` is listed.
 */
export async function waitForInboxMessageViaDrawer(page: Page, subject: string, timeout = 45_000): Promise<void> {
  await expect(async () => {
    await drawerButton(page).click();
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3_000 });
  }).toPass({ timeout });
}

/** The whole list slot (row button + its action cluster), located by subject. */
export function messageSlot(page: Page, subject: string) {
  return page.locator('.list__slot').filter({ hasText: subject });
}

/**
 * The composer. On the desktop workspace it opens as a tab whose panel is a
 * region named "Compose message" (24fb6149); the modal dialog of the same name is
 * accepted too, for layouts that still present it that way.
 */
export function composeSurface(page: Page) {
  return page
    .getByRole('region', { name: 'Compose message', exact: true })
    .or(page.getByRole('dialog', { name: 'Compose message', exact: true }));
}

/**
 * The iframe that holds the open message's CONTENT, as an element (for its
 * `sandbox` / `srcdoc`; `.contentFrame()` for what it shows).
 *
 * Since 99f51b81 the reader usually renders through a wrapper: the outer
 * `iframe.reader__frame` (sandbox="allow-scripts", opaque origin) loads only the
 * server's fixed bridge (`/api/rc/frame`, crates/mw-server/src/remote-frame.js),
 * which puts the message into a nested `iframe[title="Message content"]` with
 * `sandbox=""` and a `srcdoc`. Without the wrapper (native shells, receipt mode)
 * the outer frame itself is the `sandbox=""` srcdoc frame. Either way the frame
 * this returns is the one whose sandbox must forbid scripts.
 */
export async function readerContent(page: Page): Promise<Locator> {
  const outer = page.locator('iframe.reader__frame');
  await expect(outer).toBeVisible();
  if ((await outer.getAttribute('sandbox')) === '') return outer;
  const inner = outer.contentFrame().locator('iframe[title="Message content"]');
  await expect(inner).toBeAttached();
  return inner;
}

/**
 * The Settings surface. On the desktop workspace Settings opens as a tab whose
 * panel holds a region named "Settings" (24fb6149; its "Close settings" button
 * closes the tab); the modal dialog of the same name is accepted too, for layouts
 * that still present it that way.
 */
export function settingsSurface(page: Page) {
  return page
    .getByRole('tabpanel', { name: 'Settings' })
    .getByRole('region', { name: 'Settings', exact: true })
    .or(page.getByRole('dialog', { name: 'Settings', exact: true }));
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
  await page.getByRole('button', { name: 'Compose', exact: true }).click();
  const dialog = composeSurface(page);
  await expect(dialog).toBeVisible();
  await dialog.getByLabel('To', { exact: true }).fill(ENGINE_CREDS.selfAddress);
  await dialog.getByLabel('Subject', { exact: true }).fill(subject);
  await dialog.getByLabel('Body', { exact: true }).fill(body);
  if (opts.sendLater !== undefined) {
    await dialog.getByLabel('Send later', { exact: true }).fill(opts.sendLater);
    await expect(dialog.getByRole('button', { name: 'Schedule' })).toBeVisible();
    await dialog.getByRole('button', { name: 'Schedule' }).click();
  } else {
    await dialog.getByRole('button', { name: 'Send', exact: true }).click();
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

/**
 * Deliver a message straight into the Greenmail account over SMTP (host
 * localhost:3025), bypassing the app's compose+undo-hold. Lets a spec inject a
 * NEW delivery from an arbitrary sender (and optionally with an attachment) so
 * realtime-push / search-operator / multi-window specs don't depend on the
 * 10 s undo-send hold. The engine's watch loop ingests it like any other mail.
 */
export interface InjectAttachment {
  filename: string;
  /** MIME type (e.g. application/pdf, image/png, video/mp4). */
  contentType: string;
  /** Raw bytes as a binary string. Provide EITHER this OR `base64`. */
  content?: string;
  /** Pre-encoded base64 payload (for real binary like a PNG). */
  base64?: string;
  /** An inline Content-ID referenced by an HTML body. */
  cid?: string;
}

export async function injectViaSmtp(opts: {
  from: string;
  subject: string;
  text: string;
  to?: string;
  /** Optional sanitized-reader fixture source; plain text remains the default. */
  html?: string;
  requestReceipt?: boolean;
  /** Shorthand for a single octet-stream attachment. */
  withAttachment?: { filename: string; content: string };
  /** One or more typed attachments (image/pdf/video/…) for the viewer specs. */
  attachments?: InjectAttachment[];
}): Promise<void> {
  const host = process.env['MW_E2E_SMTP_HOST'] ?? '127.0.0.1';
  const port = Number(process.env['MW_E2E_SMTP_PORT'] ?? 3025);
  const to = opts.to ?? ENGINE_CREDS.selfAddress;

  const parts: InjectAttachment[] = opts.attachments ? [...opts.attachments] : [];
  if (opts.withAttachment !== undefined) {
    parts.push({
      filename: opts.withAttachment.filename,
      contentType: 'application/octet-stream',
      content: opts.withAttachment.content,
    });
  }

  let body: string;
  if (parts.length > 0) {
    const bound = `mwbound${Date.now()}`;
    const attachmentParts = parts
      .map((p) => {
        const b64 = p.base64 ?? Buffer.from(p.content ?? '', 'binary').toString('base64');
        // Fold the base64 into 76-char lines (RFC 2045) so long payloads parse.
        const folded = b64.replace(/(.{76})/g, '$1\r\n');
        return (
          `--${bound}\r\nContent-Type: ${p.contentType}; name="${p.filename}"\r\n` +
          `Content-Disposition: ${p.cid?'inline':'attachment'}; filename="${p.filename}"\r\n` +
          (p.cid?`Content-ID: <${p.cid}>\r\n`:'') +
          `Content-Transfer-Encoding: base64\r\n\r\n${folded}\r\n`
        );
      })
      .join('');
    body =
      `MIME-Version: 1.0\r\n` +
      `Content-Type: multipart/${parts.every(part=>part.cid)?'related':'mixed'}; boundary="${bound}"\r\n\r\n` +
      `--${bound}\r\nContent-Type: ${opts.html===undefined?'text/plain':'text/html'}; charset=utf-8\r\n\r\n${opts.html??opts.text}\r\n` +
      attachmentParts +
      `--${bound}--\r\n`;
  } else {
    body = `MIME-Version: 1.0\r\nContent-Type: ${opts.html===undefined?'text/plain':'text/html'}; charset=utf-8\r\n\r\n${opts.html??opts.text}\r\n`;
  }

  const message =
    `From: ${opts.from}\r\nTo: ${to}\r\nSubject: ${opts.subject}\r\n` +
    (opts.requestReceipt ? `Message-ID: <receipt-${Date.now()}@example.test>\r\nDisposition-Notification-To: ${opts.from}\r\n` : '') +
    `Date: ${new Date().toUTCString()}\r\n${body}`;

  const steps = [
    `EHLO mailwoman-e2e`,
    `MAIL FROM:<${extractAddr(opts.from)}>`,
    `RCPT TO:<${to}>`,
    `DATA`,
    `${message}\r\n.`,
    `QUIT`,
  ];

  await new Promise<void>((resolve, reject) => {
    const sock = net.createConnection({ host, port });
    let step = -1; // -1 = waiting for the 220 greeting
    let buf = '';
    const fail = (e: Error) => {
      sock.destroy();
      reject(e);
    };
    sock.setTimeout(15_000, () => fail(new Error('SMTP inject timed out')));
    sock.on('error', fail);
    sock.on('data', (chunk) => {
      buf += chunk.toString('utf8');
      // Act only on a complete final reply line ("NNN <text>", space after code).
      const lines = buf.split('\r\n').filter((l) => /^\d{3} /.test(l));
      if (lines.length === 0) return;
      buf = '';
      const code = Number(lines[lines.length - 1]!.slice(0, 3));
      if (code >= 400) return fail(new Error(`SMTP error: ${lines[lines.length - 1]}`));
      step += 1;
      if (step >= steps.length) {
        sock.end();
        return resolve();
      }
      sock.write(steps[step]! + '\r\n');
    });
  });
}

/** Pull the bare address out of `Name <addr>` or a bare `addr`. */
function extractAddr(input: string): string {
  const m = input.match(/<([^>]+)>/);
  return m ? m[1]! : input.trim();
}

/**
 * Forget the engine account's SERVER-STORED appearance preferences.
 *
 * Appearance is synced per account (`/api/account/appearance`, `api/prefs.ts`),
 * and every engine-mode spec signs in as the SAME account — so it is shared
 * mutable state between tests, in the same way the DCR singleton row was between
 * two Rust test binaries.
 *
 * That leaks in a way a fresh browser context does not fix, because the leak is
 * server-side. Each Playwright test gets a clean `localStorage`, so it has no
 * sync mark, and the reconcile rule for that case is "this device never synced →
 * adopt the server's" (documented in api/prefs.ts; pinned by
 * `prefs.test.ts` "adopts the account value on a device that has never synced").
 * So whatever the previously-run theming test left on the account is re-adopted
 * on the next test's first boot, and again after any reload — overwriting the
 * theme that test just picked.
 *
 * The 600 ms push debounce makes it worse rather than causing it: a test that
 * picks a theme and reloads promptly never gets its own PUT away, so it still
 * looks like a device that has never synced.
 *
 * Resetting to "the account has nothing stored" puts the reconcile on its other
 * branch — "server has nothing → push this device's set"
 * (`prefs.test.ts` "seeds the server when the account has nothing stored") — so a
 * test's own pick survives a reload. This fixes the ISOLATION rather than the
 * order: it does not depend on which specs run, or in what sequence, or on
 * whether the runner parallelises them.
 *
 * Uses its own request context (the `request` fixture), so it runs before the
 * page has navigated and cannot race the app's own boot-time reconcile.
 *
 * The DELETE carries the session's storage authority (see `engineApiSignIn`).
 */
export async function resetAccountAppearance(request: APIRequestContext): Promise<void> {
  const authority = await engineApiSignIn(request, 'the appearance reset');
  const cleared = await request.delete('/api/account/appearance', {
    headers: { 'x-mailwoman-storage-authority': authority },
  });
  expect(
    [200, 204].includes(cleared.status()),
    `DELETE /api/account/appearance ⇒ ${cleared.status()} (expected 200 or 204)`,
  ).toBe(true);
}

/**
 * Store an appearance for the engine account on the server, so the NEXT sign-in
 * in the browser starts on it.
 *
 * Seeding `localStorage` before boot does not do this any more: on every account
 * change (sign-in included) the web client resets the local set to the defaults
 * and then adopts the account's stored set, server first (`bindAppearanceToAccount`
 * in `src/api/prefs.ts`, 24fb6149), so one account never inherits another's look.
 * A spec that wants a given pack on screen after `engineLogin` stores it here.
 * `appearance` is merged over the defaults by the client (`setAppearancePrefs`).
 */
export async function setAccountAppearance(
  request: APIRequestContext,
  appearance: Record<string, unknown>,
): Promise<void> {
  const authority = await engineApiSignIn(request, 'the appearance seed');
  const put = await request.put('/api/account/appearance', {
    headers: { 'x-mailwoman-storage-authority': authority },
    data: { appearance },
  });
  expect(put.ok(), `PUT /api/account/appearance ⇒ ${put.status()} ${await put.text()}`).toBe(true);
}

/**
 * Sign `request` in to the engine account over the API and return the session's
 * storage authority. Personal-settings writes carry it in
 * `x-mailwoman-storage-authority` (0a11b581; without it the server answers 428
 * `storageAuthorityRequired`); the web client sends the value `/api/login` or
 * `/api/me` returned (`src/api/storageAuthority.ts`), and so do these helpers.
 */
async function engineApiSignIn(request: APIRequestContext, purpose: string): Promise<string> {
  const login = await postSignIn(request, '/api/login', {
    username: ENGINE_CREDS.username,
    password: ENGINE_CREDS.password,
  });
  expect(login.status(), `engine API login for ${purpose}`).toBe(200);
  let authority = ((await login.json().catch(() => null)) as { storageAuthority?: unknown } | null)?.storageAuthority;
  if (typeof authority !== 'string') {
    const me = await request.get('/api/me');
    authority = ((await me.json().catch(() => null)) as { storageAuthority?: unknown } | null)?.storageAuthority;
  }
  expect(typeof authority, 'storageAuthority from /api/login or /api/me').toBe('string');
  return authority as string;
}

/**
 * Headers for a personal-settings write a spec makes itself (`page.request`, the
 * `request` fixture, a context from `playwright.request.newContext()`), with the
 * session already signed in to the mailbox.
 *
 * Since 0a11b581 every write under `/api/settings/*`, `/api/account/*` and the
 * other personal stores must repeat the session's storage authority in
 * `x-mailwoman-storage-authority` (else 428 `storageAuthorityRequired`), as the web
 * client does (`src/api/storageAuthority.ts`). The value comes from `/api/me`.
 * CSRF is separate: the `./csrf.ts` fixture adds `X-CSRF-Token`.
 */
export async function storageAuthorityHeaders(request: APIRequestContext): Promise<Record<string, string>> {
  const me = await request.get('/api/me');
  expect(me.ok(), `GET /api/me ⇒ ${me.status()}`).toBe(true);
  const authority = ((await me.json().catch(() => null)) as { storageAuthority?: unknown } | null)?.storageAuthority;
  expect(typeof authority, 'storageAuthority from /api/me').toBe('string');
  return { 'x-mailwoman-storage-authority': authority as string };
}

/**
 * Make a composer-ribbon control reachable. Since the t38-e5 ribbon rebuild a group
 * that does not fit folds into a "More <group> commands" dropdown (on a phone most of
 * them; at 1280 px Editing and Recipients), and the ribbon can re-fold while it
 * measures itself, which closes that dropdown. Retries opening `group` until
 * `control` is visible, then returns it.
 */
export async function ribbonControl(scope: Page | Locator, group: string, control: Locator): Promise<Locator> {
  const more = scope.getByRole('button', { name: `More ${group} commands`, exact: true });
  await expect(async () => {
    if (!(await control.isVisible()) && (await more.isVisible()) && (await more.getAttribute('aria-expanded')) !== 'true') {
      await more.click({ timeout: 1_000 });
    }
    await expect(control).toBeVisible({ timeout: 1_000 });
  }).toPass({ timeout: 15_000 });
  return control;
}
