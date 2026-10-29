import { test, expect, type Page, type Request } from './csrf.ts';
import net from 'node:net';
import {
  ENGINE_CREDS,
  engineLogin,
  injectViaSmtp,
  messageRow,
  settingsSurface,
  sidebarInbox,
  waitForInboxMessage,
} from './helpers.ts';
import { uid } from './pim-helpers.ts';

/**
 * Read state, bulk actions and the keymap, end to end (26.20 t28-e16).
 *
 * Every case reads its outcome from the MAIL SERVER: the engine account's
 * mailboxes on Greenmail, over IMAP, by this file. Nothing about the result is
 * taken from the app except what the user would see (the row, the badge).
 *
 * Greenmail advertises no SPECIAL-USE, so its account has no folder with the
 * archive role. The bulk case therefore moves to a folder this file creates
 * (the same batched relocation the Archive button uses), and the keymap case
 * shows `e` reaching the archive action through the "No Archive folder" notice
 * the action gives on such an account, and not reaching it with shortcuts off.
 */

test.describe.configure({ mode: 'serial' });

const IMAP_HOST = process.env['MW_E2E_IMAP_HOST'] ?? '127.0.0.1';
const IMAP_PORT = Number(process.env['MW_E2E_IMAP_PORT'] ?? 3143);
/** The preferences this file sets (see `storePrefs`). */
const PREF_KEYS = ['markRead', 'markReadDelayMs', 'keyboardPreset'];

// ── The mail server, directly ────────────────────────────────────────────────

/** Run IMAP commands as the engine account; resolves to each command's full reply. */
function imap(commands: string[]): Promise<string[]> {
  return new Promise<string[]>((resolve, reject) => {
    const sock = net.createConnection({ host: IMAP_HOST, port: IMAP_PORT });
    sock.setEncoding('latin1');
    const all = [`LOGIN "${ENGINE_CREDS.username}" "${ENGINE_CREDS.password}"`, ...commands, 'LOGOUT'];
    const replies: string[] = [];
    let step = 0; // 0: waiting for the greeting
    let buf = '';
    const fail = (e: Error): void => {
      sock.destroy();
      reject(e);
    };
    const send = (): void => {
      step += 1;
      buf = '';
      sock.write(`a${step} ${all[step - 1]!}\r\n`);
    };
    sock.setTimeout(15_000, () => fail(new Error('IMAP timed out')));
    sock.on('error', fail);
    sock.on('data', (chunk: string) => {
      buf += chunk;
      if (step === 0) {
        if (buf.includes('\r\n')) send();
        return;
      }
      const done = new RegExp(`(^|\\r\\n)a${step} (OK|NO|BAD)[^\\r\\n]*\\r\\n$`).exec(buf);
      if (done === null) return;
      if (step > 1 && step <= all.length - 1) replies.push(buf);
      if (step === all.length) {
        sock.end();
        resolve(replies);
        return;
      }
      if (done[2] !== 'OK' && step === 1) return fail(new Error(`IMAP login failed: ${buf}`));
      send();
    });
  });
}

/** The FLAGS of every message in `folder` whose Subject contains `subject`. */
async function flagsOf(subject: string, folder = 'INBOX'): Promise<string[]> {
  const [, search] = await imap([`EXAMINE "${folder}"`, `SEARCH SUBJECT "${subject}"`]);
  const ids = (/\* SEARCH([^\r\n]*)/.exec(search ?? '')?.[1] ?? '').trim().split(/\s+/).filter((s) => s !== '');
  if (ids.length === 0) return [];
  const [, , fetch] = await imap([`EXAMINE "${folder}"`, `SEARCH SUBJECT "${subject}"`, `FETCH ${ids.join(',')} (FLAGS)`]);
  return [...(fetch ?? '').matchAll(/FLAGS \(([^)]*)\)/g)].map((m) => m[1] ?? '');
}

/** How many messages in `folder` have `subject` in their Subject. */
async function countIn(folder: string, subject: string): Promise<number> {
  const [, search] = await imap([`EXAMINE "${folder}"`, `SEARCH SUBJECT "${subject}"`]);
  return (/\* SEARCH([^\r\n]*)/.exec(search ?? '')?.[1] ?? '').trim().split(/\s+/).filter((s) => s !== '').length;
}

// ── The app ──────────────────────────────────────────────────────────────────

/**
 * The Inbox's unread count from its badge (0 when the badge is not shown). The
 * badge reads "unread / total" by default since 12445b55 (folder count modes), so
 * the unread count is the number before the slash.
 */
async function inboxUnread(page: Page): Promise<number> {
  const badge = sidebarInbox(page).locator('.sidebar__badge');
  if ((await badge.count()) === 0) return 0;
  const text = (await badge.textContent()) ?? '';
  return Number(text.split('/')[0]!.trim() || '0');
}

/**
 * Store `prefs` as the signed-in account's preferences, the way Settings stores
 * them. Since 24fb6149 a signed-in client keeps these on the account
 * (`/api/account/preferences`, src/screens/Settings/prefs.ts `loadPrefs`) and
 * reads `localStorage` only while signed out. The keys this file sets are
 * dropped first, so `{}` means their defaults; the account's other preferences
 * are kept. Resolves false when the page is not signed in.
 */
async function storePrefs(page: Page, prefs: Record<string, unknown>): Promise<boolean> {
  const me = await page.request.get('/api/me');
  if (!me.ok()) return false;
  const authority = ((await me.json()) as { storageAuthority?: unknown }).storageAuthority;
  expect(typeof authority, 'storageAuthority from /api/me').toBe('string');
  const headers = { 'x-mailwoman-storage-authority': authority as string };
  const current = await page.request.get('/api/account/preferences', { headers });
  expect(current.ok(), `GET /api/account/preferences ⇒ ${current.status()}`).toBe(true);
  const etag = current.headers()['etag'];
  expect(etag, 'the preferences carry an ETag').toBeTruthy();
  const next = { ...((await current.json()) as { preferences: Record<string, unknown> }).preferences };
  for (const key of PREF_KEYS) delete next[key];
  const put = await page.request.put('/api/account/preferences', {
    headers: { ...headers, 'if-match': etag! },
    data: { version: 1, preferences: { ...next, ...prefs } },
  });
  expect(put.ok(), `PUT /api/account/preferences ⇒ ${put.status()} ${await put.text()}`).toBe(true);
  return true;
}

/** Store `prefs` on the account, then reload so the client adopts them. */
async function setPrefs(page: Page, prefs: Record<string, unknown>): Promise<void> {
  await storePrefs(page, prefs);
  const adopted = page.waitForResponse(
    (res) => res.request().method() === 'GET' && new URL(res.url()).pathname === '/api/account/preferences',
  );
  await page.reload();
  expect((await adopted).ok()).toBe(true);
  await expect(sidebarInbox(page)).toBeVisible();
}

// The account is shared by every engine spec: leave its read and keyboard
// policy at the defaults whatever a case set (through the API or Settings).
test.afterEach(async ({ page }) => {
  await storePrefs(page, {}).catch(() => undefined);
});

/** Record every JMAP request that carries an `Email/set`. */
function emailSets(page: Page): Request[] {
  const seen: Request[] = [];
  page.on('request', (req) => {
    if (req.method() === 'POST' && (req.postData() ?? '').includes('"Email/set"')) seen.push(req);
  });
  return seen;
}

test('opening an unread message sets \\Seen on the server after the delay, the badge drops, and Mark unread clears it', async ({ page }) => {
  const subject = `read-state-${uid()}`;
  await injectViaSmtp({ from: 'alice@example.org', subject, text: 'read me' });
  await engineLogin(page);
  await waitForInboxMessage(page, subject);

  // Precondition: unread in the list and on the server.
  await expect(messageRow(page, subject)).toHaveClass(/list__row--unread/);
  expect(await flagsOf(subject)).toEqual([expect.not.stringContaining('\\Seen')]);
  const before = await inboxUnread(page);
  expect(before).toBeGreaterThan(0);

  // Control: with automatic mark-read off, opening the message (which fetches
  // its body from the server) leaves it unread. So the \Seen below is written by
  // the policy, not by the fetch.
  await setPrefs(page, { markRead: 'never' });
  await messageRow(page, subject).click();
  await expect(page.locator('.reader__subject')).toHaveText(subject);
  await page.waitForTimeout(3_000);
  expect(await flagsOf(subject)).toEqual([expect.not.stringContaining('\\Seen')]);
  await expect(messageRow(page, subject)).toHaveClass(/list__row--unread/);

  // With the default policy, a message that stays open is marked read.
  await setPrefs(page, {});
  await messageRow(page, subject).click(); // open it again under the new policy
  await expect.poll(async () => (await flagsOf(subject)).join(' '), { timeout: 15_000 }).toContain('\\Seen');
  await expect(messageRow(page, subject)).not.toHaveClass(/list__row--unread/);
  await expect.poll(() => inboxUnread(page)).toBe(before - 1);

  // Mark unread from the reader clears the flag on the server.
  await page.getByTestId('reader-toggle-read').click();
  await expect.poll(async () => (await flagsOf(subject)).join(' '), { timeout: 15_000 }).not.toContain('\\Seen');
  await expect(messageRow(page, subject)).toHaveClass(/list__row--unread/);
  await expect.poll(() => inboxUnread(page)).toBe(before);
  // And it stays unread: the pending mark-read for the open message was cancelled.
  await page.waitForTimeout(2_500);
  expect((await flagsOf(subject)).join(' ')).not.toContain('\\Seen');

  // Flag sets \Flagged on the server.
  await page.getByTestId('reader-flag').click();
  await expect.poll(async () => (await flagsOf(subject)).join(' '), { timeout: 15_000 }).toContain('\\Flagged');
  await expect(messageRow(page, subject).getByTestId('row-flagged')).toBeVisible();
});

test('three selected messages are marked read, then moved, each with one request, and the server has all three', async ({ page }) => {
  const tag = uid();
  const folder = `Bulk-${tag}`;
  const subjects = [1, 2, 3].map((n) => `bulk-${tag}-${n}`);
  await imap([`CREATE "${folder}"`]);
  for (const subject of subjects) await injectViaSmtp({ from: 'bob@example.org', subject, text: 'bulk' });
  await engineLogin(page);
  for (const subject of subjects) await waitForInboxMessage(page, subject);

  // Precondition: all three unread in the INBOX, none in the folder.
  expect(await countIn('INBOX', `bulk-${tag}`)).toBe(3);
  expect(await countIn(folder, `bulk-${tag}`)).toBe(0);
  for (const subject of subjects) expect((await flagsOf(subject)).join(' ')).not.toContain('\\Seen');

  // Select: Ctrl+click the first, Shift+click the last (the range covers the middle).
  await messageRow(page, subjects[2]!).click({ modifiers: ['ControlOrMeta'] });
  await messageRow(page, subjects[0]!).click({ modifiers: ['Shift'] });
  const bar = page.getByRole('toolbar', { name: 'Selection actions' });
  await expect(bar.getByTestId('bulk-count')).toHaveText('3 selected');

  const sets = emailSets(page);
  await bar.getByRole('button', { name: 'Mark read', exact: true }).click();
  for (const subject of subjects) {
    await expect.poll(async () => (await flagsOf(subject)).join(' '), { timeout: 15_000 }).toContain('\\Seen');
  }
  expect(sets).toHaveLength(1);

  sets.length = 0;
  // The floating selection bar (t58): Move opens a menu of folders above the bar.
  await bar.getByRole('button', { name: 'Move to folder', exact: true }).click();
  await page.getByRole('menu', { name: 'Move to folder' }).getByRole('menuitem', { name: folder, exact: true }).click();
  await expect.poll(() => countIn(folder, `bulk-${tag}`), { timeout: 15_000 }).toBe(3);
  expect(await countIn('INBOX', `bulk-${tag}`)).toBe(0);
  expect(sets).toHaveLength(1);
  const update = (JSON.parse(sets[0]!.postData()!) as { methodCalls: [string, { update?: Record<string, unknown> }, string][] })
    .methodCalls.find((c) => c[0] === 'Email/set')?.[1].update;
  expect(Object.keys(update ?? {})).toHaveLength(3);
  for (const subject of subjects) await expect(messageRow(page, subject)).toHaveCount(0);
});

test('with the Gmail preset chosen in Settings, e runs Archive; with shortcuts off it does nothing', async ({ page }) => {
  const subject = `keymap-${uid()}`;
  await injectViaSmtp({ from: 'carol@example.org', subject, text: 'keys' });
  await engineLogin(page);
  await waitForInboxMessage(page, subject);

  // Gmail, through the Settings screen.
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  // Settings is sectioned; the presets are under "Keyboard & display".
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Keyboard & display', exact: true }).click();
  const kbd = page.getByRole('group', { name: 'Shortcut preset', exact: true });
  await kbd.getByRole('button', { name: 'Gmail' }).click();
  await expect(kbd.getByRole('button', { name: 'Gmail' })).toHaveAttribute('aria-pressed', 'true');
  await settingsSurface(page).getByRole('button', { name: 'Close settings' }).click();

  await messageRow(page, subject).click();
  await expect(page.locator('.reader__subject')).toHaveText(subject);
  await page.locator('.reader__subject').click(); // focus on the page, not in a field
  await page.keyboard.press('e');
  // This account has no archive-role folder (see the file header): the archive
  // action says so. Without the shortcut nothing would.
  await expect(page.getByText('No Archive folder')).toBeVisible();
  await expect(page.getByText('No Archive folder')).toBeHidden({ timeout: 15_000 });

  // Shortcuts off: the same key reaches no action.
  await setPrefs(page, { keyboardPreset: 'off' });
  // The reload drops focus: put it back where the first press was made.
  await messageRow(page, subject).click();
  await expect(page.locator('.reader__subject')).toHaveText(subject);
  await page.locator('.reader__subject').click();
  await page.keyboard.press('e');
  await page.waitForTimeout(1_500);
  await expect(page.getByText('No Archive folder')).toHaveCount(0);
  await expect(messageRow(page, subject)).toHaveCount(1);
  expect(await countIn('INBOX', subject)).toBe(1);
});
