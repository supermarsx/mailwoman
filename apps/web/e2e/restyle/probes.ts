import type { APIRequestContext, Locator, Page } from '@playwright/test';
import { test as csrfTest, expect } from '../csrf.ts';
import {
  drawerButton,
  engineLogin,
  enginePhoneLogin,
  messageRow,
  messageSlot,
  resetAccountAppearance,
  sidebarInbox,
} from '../helpers.ts';

/**
 * Screens and probes for the t32 restyle harness (README.md in this folder).
 *
 * A probe names elements that exist at T32-BASE (ff52430) and that t32 does not
 * remove: located by role and accessible name, by test id, or by a BEM class
 * written in markup (`.list__row`). Never by a vanilla-extract class (`_1avqldf0`):
 * those hashes change whenever a `.css.ts` file changes. Admin probes use roles
 * only, by position where a name may change (the admin track rebuilds that screen).
 *
 * Each probe has a kind (`control`, `text`, `container`) and an `interactive` flag;
 * `compare.mjs` applies its thresholds by kind and flag. `max` measures the first N
 * matches, keyed `<probe>#<index>`.
 */

export type ViewportName = 'desktop' | 'phone';
export type ThemeName = 'dark' | 'light';

export interface RestyleOptions {
  viewportName: ViewportName;
  theme: ThemeName;
}

export type ProbeKind = 'control' | 'text' | 'container';

export interface Probe {
  id: string;
  kind: ProbeKind;
  interactive: boolean;
  locate: (page: Page) => Locator;
  /** Measure the first N matches (default 1). */
  max?: number;
}

export interface Screen {
  id: string;
  /** Group tag for `--grep`, for example `@mail`. */
  tag: string;
  viewports: ViewportName[];
  /** Reach the screen. The page already carries the theme seed and the fixed clock. */
  open: (page: Page, ctx: ScreenContext) => Promise<void>;
  probes: Probe[];
  /** Count fully visible list rows (`row` inside `scroller`, both inside the viewport). */
  rows?: { row: string; scroller: string };
  /** Selectors axe and the non-text checks skip (untrusted message HTML). */
  exclude?: string[];
  /**
   * False when the screen does not apply the theme packs at T32-BASE (the admin panel): the
   * `data-theme` check is skipped and the value found is recorded instead.
   */
  themed?: boolean;
  /** Runs after the screen was measured, to leave the account as it was. */
  close?: (page: Page) => Promise<void>;
}

export interface ScreenContext {
  viewportName: ViewportName;
  theme: ThemeName;
  request: APIRequestContext;
}

/** The fixed browser clock. Message dates come from the server and still vary. */
export const FIXED_NOW = new Date('2027-03-10T10:00:00Z');

/** The message the reader screens open: seeded read (seed.mjs), so opening it changes nothing. */
export const READER_SUBJECT = 'Re: Contract renewal 25';
/** The row the toast screen acts on (Archive with no Archive folder shows an error toast). */
export const TOAST_SUBJECT = 'Weekly status 28';

const PIM = {
  task: 'T32 task: file the report',
  note: 'T32 note: meeting minutes',
  contact: 'Ada T32 Marsh',
  contactEmail: 'ada.t32@example.net',
  event: 'T32 design review',
};

const ADMIN_USER = { username: 't32user', domain: 'example.org' };

// ── probe builders ──────────────────────────────────────────────────────────

function control(id: string, locate: (p: Page) => Locator, max?: number): Probe {
  return { id, kind: 'control', interactive: true, locate, ...(max ? { max } : {}) };
}
function text(id: string, locate: (p: Page) => Locator, max?: number): Probe {
  return { id, kind: 'text', interactive: false, locate, ...(max ? { max } : {}) };
}
function container(id: string, locate: (p: Page) => Locator, max?: number): Probe {
  return { id, kind: 'container', interactive: false, locate, ...(max ? { max } : {}) };
}
/** A list row: a control whose height is the row height and whose width follows the pane. */
function row(id: string, locate: (p: Page) => Locator, max?: number): Probe {
  return { id, kind: 'container', interactive: true, locate, ...(max ? { max } : {}) };
}

const btn = (p: Page | Locator, name: string | RegExp, exact = true) =>
  typeof name === 'string' ? p.getByRole('button', { name, exact }) : p.getByRole('button', { name });

// ── shared probe sets ───────────────────────────────────────────────────────

const listProbes: Probe[] = [
  container('list.region', (p) => p.getByRole('region', { name: 'Messages' })),
  row('list.row', (p) => p.locator('.list__row'), 15),
  text('list.sender', (p) => p.locator('.list__sender'), 15),
  text('list.subject', (p) => p.locator('.list__subject'), 15),
  text('list.date', (p) => p.locator('.list__date'), 15),
  text('list.preview', (p) => p.locator('.list__preview'), 12),
];

const sidebarProbes: Probe[] = [
  container('sidebar', (p) => p.locator('aside.sidebar')),
  text('sidebar.brand', (p) => p.locator('.sidebar__brand')),
  text('sidebar.user', (p) => p.locator('.sidebar__account')),
  control('sidebar.settings', (p) => btn(p.locator('aside.sidebar'), 'Settings')),
  control('sidebar.compose', (p) => btn(p.locator('aside.sidebar'), 'Compose')),
  control('sidebar.mailbox', (p) => p.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button'), 4),
  text('sidebar.badge', (p) => p.locator('.sidebar__badge')),
  control('sidebar.app', (p) => p.getByRole('navigation', { name: 'Apps' }).getByRole('button'), 5),
  control('sidebar.logout', (p) => btn(p, 'Log out')),
];

const mailToolbarProbes: Probe[] = [
  container('tabs.strip', (p) => p.getByRole('tablist', { name: 'Open tabs' })),
  control('tabs.mail', (p) => p.getByRole('tab', { name: 'Mail' })),
  control('tabs.pin', (p) => btn(p, 'Unpin Mail')),
  control('tabs.tearoff', (p) => btn(p, 'Open Mail in a new window')),
  control('tabs.close', (p) => btn(p, 'Close Mail')),
  control('search.input', (p) => p.getByRole('searchbox', { name: 'Search mail' })),
  control('search.submit', (p) => btn(p.getByRole('search'), 'Search')),
  control('inbox.focused', (p) => btn(p, 'Focused inbox')),
  text('pane.label', (p) => p.getByRole('group', { name: 'View options' }).getByText('Reading pane')),
  control('pane.option', (p) => p.getByRole('group', { name: 'View options' }).getByRole('button'), 3),
];

const readerProbes: Probe[] = [
  container('reader', (p) => p.getByRole('region', { name: 'Message' })),
  control('reader.back', (p) => btn(p.getByRole('region', { name: 'Message' }), /Back/)),
  text('reader.subject', (p) => p.getByRole('region', { name: 'Message' }).getByRole('heading', { level: 2 })),
  text('reader.meta', (p) => p.locator('.reader__meta')),
  container('reader.toolbar', (p) => p.getByRole('toolbar', { name: 'Message actions' })),
  control('reader.action', (p) => p.getByRole('toolbar', { name: 'Message actions' }).getByRole('button'), 10),
  control('reader.level', (p) => p.getByRole('radiogroup', { name: 'Message security level' }).getByRole('radio'), 3),
  container('reader.frame', (p) => p.locator('iframe.reader__frame')),
];

const composeProbes: Probe[] = [
  container('compose', (p) => p.getByRole('dialog', { name: 'Compose message' })),
  text('compose.title', (p) => p.getByRole('dialog', { name: 'Compose message' }).getByRole('heading', { name: 'New message' })),
  control('compose.drafts', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Drafts')),
  control('compose.recall', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Recall')),
  control('compose.close', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Close')),
  control('compose.from', (p) => p.getByRole('dialog', { name: 'Compose message' }).getByRole('combobox', { name: 'From' })),
  control('compose.to', (p) => p.getByRole('dialog', { name: 'Compose message' }).getByRole('textbox', { name: 'To', exact: true })),
  control('compose.cc', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Add Cc or Bcc')),
  control('compose.subject', (p) =>
    p.getByRole('dialog', { name: 'Compose message' }).getByRole('textbox', { name: 'Subject', exact: true }),
  ),
  control('compose.plain', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Plain text')),
  control('compose.format', (p) => p.getByRole('toolbar', { name: 'Formatting' }).getByRole('button'), 8),
  control('compose.body', (p) => p.getByRole('dialog', { name: 'Compose message' }).getByRole('textbox', { name: 'Body' })),
  control('compose.attach', (p) => btn(p.getByRole('dialog', { name: 'Compose message' }), 'Attach a file')),
  text('compose.label', (p) => p.getByRole('dialog', { name: 'Compose message' }).getByText(/^(From|To|Subject|Body)$/), 4),
];

const settingsDialog = (p: Page) => p.getByRole('dialog', { name: 'Settings' });
const settingsProbes: Probe[] = [
  container('settings', settingsDialog),
  text('settings.title', (p) => settingsDialog(p).getByRole('heading', { name: 'Appearance' })),
  control('settings.close', (p) => btn(settingsDialog(p), 'Close settings')),
  control('settings.mode', (p) => settingsDialog(p).getByRole('group', { name: 'When to switch' }).getByRole('button'), 3),
  control('settings.theme-card', (p) => settingsDialog(p).getByRole('button', { name: /^(Light|Dark|Slate|Ocean|Plum|Grove|AMOLED|High contrast)/ }), 13),
  text('settings.family', (p) => settingsDialog(p).getByText(/^(Default|Slate|Ocean|Plum|Grove|High contrast)$/), 6),
  control('settings.forget', (p) => btn(settingsDialog(p), 'Forget the appearance saved for my account')),
  control('settings.density', (p) => settingsDialog(p).getByRole('group', { name: 'Density' }).getByRole('button'), 3),
  control('settings.accent', (p) => settingsDialog(p).getByRole('group', { name: 'Accent' }).getByRole('button'), 7),
  control('settings.font', (p) => settingsDialog(p).getByRole('group', { name: 'Interface font' }).getByRole('button'), 4),
  control('settings.layout', (p) => settingsDialog(p).getByRole('group', { name: 'Layout' }).getByRole('button'), 2),
  text('settings.section', (p) => settingsDialog(p).getByRole('heading', { level: 2 }), 10),
  control('settings.password', (p) => settingsDialog(p).getByRole('form', { name: 'Change password' }).getByRole('textbox'), 3),
  control('settings.password-submit', (p) => btn(settingsDialog(p), 'Change password')),
  control('settings.checkbox', (p) => settingsDialog(p).getByRole('checkbox'), 4),
];

// ── open helpers ────────────────────────────────────────────────────────────

async function signedIn(page: Page, ctx: ScreenContext): Promise<void> {
  if (ctx.viewportName === 'phone') await enginePhoneLogin(page);
  else await engineLogin(page);
  await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 45_000 });
}

async function openReader(page: Page, ctx: ScreenContext): Promise<void> {
  await signedIn(page, ctx);
  await messageRow(page, READER_SUBJECT).click();
  await expect(page.getByRole('region', { name: 'Message' }).getByRole('heading', { name: READER_SUBJECT })).toBeVisible();
  await expect(page.locator('iframe.reader__frame')).toBeVisible();
}

async function openModule(page: Page, ctx: ScreenContext, id: 'calendar' | 'contacts' | 'tasks' | 'notes') {
  await signedIn(page, ctx);
  await page.getByTestId(`nav-${id}`).click();
  await expect(page.locator(`[data-module="${id}"]`)).toBeVisible();
}

/** Create `PIM.*` items through the UI once per stack; later runs find them. */
async function ensureTask(page: Page): Promise<void> {
  const tasks = page.getByRole('list', { name: 'Tasks' });
  await expect(tasks).toBeVisible();
  await page.waitForTimeout(500);
  if ((await tasks.getByText(PIM.task, { exact: true }).count()) === 0) {
    await page.getByRole('textbox', { name: 'New task title' }).fill(PIM.task);
    await btn(page, 'Add').click();
  }
  await expect(tasks.getByText(PIM.task, { exact: true })).toBeVisible();
}

async function ensureNote(page: Page): Promise<void> {
  const list = page.getByRole('listbox', { name: 'Notes' });
  await expect(list).toBeVisible();
  await page.waitForTimeout(500);
  if ((await list.getByRole('option', { name: PIM.note }).count()) === 0) {
    await btn(page, '+ New note').click();
    const title = page.getByRole('textbox', { name: 'Note title' });
    await expect(title).toHaveValue('Untitled note');
    await title.fill(PIM.note);
  }
  await list.getByRole('option', { name: PIM.note }).click();
  await expect(page.getByRole('textbox', { name: 'Note title' })).toHaveValue(PIM.note);
}

async function ensureContact(page: Page): Promise<void> {
  const region = page.getByRole('region', { name: 'Contacts' });
  await page.waitForTimeout(500);
  if ((await region.getByText(PIM.contact).count()) === 0) {
    await btn(region, 'New contact').click();
    const form = region.getByRole('form', { name: 'New contact' });
    await form.getByLabel('Full name').fill(PIM.contact);
    await form.getByLabel('Email 1', { exact: true }).fill(PIM.contactEmail);
    await btn(form, 'Save').click();
    await expect(form).toBeHidden();
  }
  await region.getByText(PIM.contact).first().click();
}

async function ensureEvent(page: Page): Promise<void> {
  const grid = page.getByRole('grid', { name: 'Time grid' });
  await expect(grid).toBeVisible();
  await page.waitForTimeout(500);
  if ((await page.getByText(PIM.event).count()) === 0) {
    await btn(page, 'New event').click();
    const dialog = page.getByRole('dialog', { name: 'New event' });
    await expect(dialog).toBeVisible();
    await dialog.getByLabel('Title').fill(PIM.event);
    // The default start is "now" on the fixed clock: Wednesday 2027-03-10, inside the week shown.
    await btn(dialog, 'Save').click();
    await expect(dialog).toBeHidden();
  }
  await expect(page.getByText(PIM.event).first()).toBeVisible();
}

async function adminSignIn(page: Page): Promise<void> {
  await page.goto('/admin');
  await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
  const user = process.env['MW_ADMIN_USER'];
  const password = process.env['MW_ADMIN_PASSWORD'];
  if (!user || !password) throw new Error('MW_ADMIN_USER / MW_ADMIN_PASSWORD are not set (T32_STACK_ENV)');
  const main = page.getByRole('main');
  await main.getByRole('textbox').nth(0).fill(user);
  await main.getByRole('textbox').nth(1).fill(password);
  await page.getByRole('button', { name: /sign in/i }).click();
  await expect(page.getByRole('navigation').first()).toBeVisible();
}

// ── screens ─────────────────────────────────────────────────────────────────

const adminNav = (p: Page) => p.getByRole('navigation').first();
/**
 * A section entry in the admin nav: a button at T32-BASE, a link in the rebuilt console
 * (t32-e8, sections at /admin#/<section>). Either role counts, in document order.
 */
const adminNavItem = (p: Page, name?: string) =>
  name === undefined
    ? adminNav(p).getByRole('link').or(adminNav(p).getByRole('button'))
    : adminNav(p).getByRole('link', { name, exact: true }).or(adminNav(p).getByRole('button', { name, exact: true }));

export const SCREENS: Screen[] = [
  {
    id: 'login',
    tag: '@login',
    viewports: ['desktop', 'phone'],
    open: async (page) => {
      await page.goto('/');
      await expect(btn(page, 'Sign in')).toBeVisible();
    },
    probes: [
      container('login.main', (p) => p.getByRole('main')),
      container('login.card', (p) => p.getByRole('form', { name: 'Sign in' })),
      text('login.title', (p) => p.getByRole('heading', { level: 1 })),
      text('login.label', (p) => p.getByRole('form', { name: 'Sign in' }).locator('label'), 2),
      control('login.email', (p) => p.getByRole('textbox', { name: 'Email or username' })),
      control('login.password', (p) => p.getByRole('textbox', { name: 'Password' })),
      control('login.submit', (p) => btn(p, 'Sign in')),
    ],
  },
  {
    id: 'boot-error',
    tag: '@shell',
    viewports: ['desktop', 'phone'],
    open: async (page) => {
      await page.route('**/api/me', (r) => r.fulfill({ status: 500, contentType: 'text/plain', body: 'error' }));
      await page.goto('/');
      await expect(page.getByRole('alert')).toBeVisible();
      await expect(btn(page.getByRole('alert'), 'Retry')).toBeVisible();
    },
    probes: [
      container('boot.card', (p) => p.getByRole('alert')),
      text('boot.message', (p) => p.getByRole('alert').getByRole('paragraph').first()),
      control('boot.retry', (p) => btn(p.getByRole('alert'), 'Retry')),
    ],
  },
  {
    id: 'mailbox',
    tag: '@mail',
    viewports: ['desktop'],
    open: signedIn,
    probes: [...sidebarProbes, ...mailToolbarProbes, ...listProbes, text('reader.empty', (p) => p.locator('.reader__empty'))],
    rows: { row: '.list__row', scroller: '.list__scroll' },
  },
  {
    id: 'reader',
    tag: '@mail',
    viewports: ['desktop'],
    open: openReader,
    probes: [...listProbes, row('list.row-active', (p) => p.locator('.list__row--active')), ...readerProbes],
    rows: { row: '.list__row', scroller: '.list__scroll' },
    exclude: ['iframe.reader__frame'],
  },
  {
    id: 'compose',
    tag: '@compose',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      await btn(page.locator('aside.sidebar'), 'Compose').click();
      await expect(page.getByRole('dialog', { name: 'Compose message' })).toBeVisible();
    },
    probes: composeProbes,
  },
  {
    id: 'settings',
    tag: '@settings',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      await btn(page, 'Settings').click();
      await expect(settingsDialog(page)).toBeVisible();
      await expect(settingsDialog(page).getByRole('heading', { name: 'Appearance' })).toBeVisible();
    },
    probes: settingsProbes,
  },
  {
    id: 'toast',
    tag: '@shell',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      const slot = messageSlot(page, TOAST_SUBJECT);
      await slot.hover();
      // The seeded account has no Archive folder, so Archive leaves the message
      // where it is and shows an error toast: a toast that changes nothing.
      await btn(slot, 'Archive').click();
      await expect(page.locator('.toast')).toBeVisible();
    },
    probes: [text('toast', (p) => p.locator('.toast'))],
  },
  {
    id: 'empty-folder',
    tag: '@mail',
    viewports: ['desktop', 'phone'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      if (ctx.viewportName === 'phone') await drawerButton(page).click();
      await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Outbox' }).click();
      await expect(page.locator('.outbox')).toBeVisible();
    },
    probes: [
      container('outbox', (p) => p.getByRole('region', { name: 'Outbox' })),
      text('outbox.title', (p) => p.getByRole('region', { name: 'Outbox' }).getByRole('heading', { level: 2 })),
      control('outbox.refresh', (p) => btn(p.getByRole('region', { name: 'Outbox' }), 'Refresh')),
      text('outbox.empty', (p) => p.locator('.outbox__empty')),
    ],
  },
  {
    id: 'calendar',
    tag: '@pim',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await openModule(page, ctx, 'calendar');
      await ensureEvent(page);
    },
    probes: [
      container('cal', (p) => p.getByRole('region', { name: 'Calendar' })),
      text('cal.title', (p) => p.getByRole('region', { name: 'Calendar' }).getByText('March 2027', { exact: true })),
      control('cal.prev', (p) => btn(p, 'Previous')),
      control('cal.today', (p) => btn(p, 'Today')),
      control('cal.next', (p) => btn(p, 'Next')),
      control('cal.view', (p) => p.getByRole('tablist', { name: 'Calendar views' }).getByRole('tab'), 9),
      control('cal.new', (p) => btn(p, 'New event')),
      control('cal.quick', (p) => p.getByRole('textbox', { name: 'Quick add event' })),
      control('cal.quick-add', (p) => btn(p, 'Quick add')),
      control('cal.import', (p) => btn(p.getByRole('region', { name: 'Calendar' }), 'Import')),
      control('cal.export', (p) => btn(p.getByRole('region', { name: 'Calendar' }), 'Export')),
      container('cal.side', (p) => p.getByRole('complementary', { name: 'Calendars' })),
      text('cal.side-heading', (p) => p.getByRole('complementary', { name: 'Calendars' }).getByRole('heading', { level: 3 }), 4),
      control('cal.toggle', (p) => p.getByRole('checkbox', { name: 'Toggle Calendar' })),
      control('cal.share', (p) => btn(p, 'Share Calendar')),
      control('cal.add', (p) => btn(p, 'Add calendar')),
      control('cal.filter', (p) => p.getByRole('textbox', { name: 'Filter by category' })),
      control('cal.url', (p) => p.getByRole('textbox', { name: 'Calendar URL' })),
      control('cal.subscribe', (p) => btn(p, 'Subscribe')),
      control('cal.holidays', (p) => p.getByRole('combobox', { name: 'Subscribe to holidays' })),
      container('cal.grid', (p) => p.getByRole('grid', { name: 'Time grid' })),
      container('cal.day', (p) => p.getByRole('gridcell'), 7),
      text('cal.event', (p) => p.getByText(PIM.event).first()),
    ],
  },
  {
    id: 'contacts',
    tag: '@pim',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await openModule(page, ctx, 'contacts');
      await ensureContact(page);
    },
    probes: [
      container('contacts', (p) => p.getByRole('region', { name: 'Contacts' })),
      container('contacts.side', (p) => p.getByRole('complementary', { name: 'Address books and groups' })),
      text('contacts.heading', (p) => p.getByRole('complementary', { name: 'Address books and groups' }).getByRole('heading', { level: 2 }), 3),
      control('contacts.book', (p) => p.getByRole('complementary', { name: 'Address books and groups' }).getByRole('button'), 5),
      control('contacts.search', (p) => p.getByRole('searchbox', { name: 'Search contacts' })),
      control('contacts.new', (p) => btn(p, 'New contact')),
      control('contacts.dupes', (p) => btn(p, 'Find duplicates')),
      text('contacts.item', (p) => p.getByRole('region', { name: 'Contacts' }).getByText(PIM.contact).first()),
      control('contacts.export', (p) => p.getByRole('button', { name: /^Export (vCard|CSV)$/ }), 2),
    ],
  },
  {
    id: 'tasks',
    tag: '@pim',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await openModule(page, ctx, 'tasks');
      await ensureTask(page);
    },
    probes: [
      container('tasks', (p) => p.getByRole('region', { name: 'Tasks' })),
      control('tasks.list', (p) => p.getByRole('navigation', { name: 'Task lists' }).getByRole('button'), 3),
      text('tasks.heading', (p) => p.getByRole('navigation', { name: 'Task lists' }).getByRole('heading', { level: 2 })),
      control('tasks.title', (p) => p.getByRole('textbox', { name: 'New task title' })),
      control('tasks.add', (p) => btn(p, 'Add')),
      container('tasks.item', (p) => p.getByRole('list', { name: 'Tasks' }).getByRole('listitem'), 1),
      control('tasks.complete', (p) => p.getByRole('checkbox', { name: `Complete ${PIM.task}` })),
      text('tasks.item-title', (p) => p.getByRole('list', { name: 'Tasks' }).getByText(PIM.task, { exact: true })),
      control('tasks.edit', (p) => btn(p, `Edit ${PIM.task}`)),
      control('tasks.convert-field', (p) => p.getByRole('textbox', { name: /to convert$/ }), 2),
      control('tasks.convert', (p) => p.getByRole('button', { name: /→ task$/ }), 2),
    ],
  },
  {
    id: 'notes',
    tag: '@pim',
    viewports: ['desktop'],
    open: async (page, ctx) => {
      await openModule(page, ctx, 'notes');
      await ensureNote(page);
    },
    probes: [
      container('notes', (p) => p.getByRole('region', { name: 'Notes' })),
      control('notes.new', (p) => btn(p, '+ New note')),
      control('notes.search', (p) => p.getByRole('searchbox', { name: 'Search notes' })),
      container('notes.list', (p) => p.getByRole('listbox', { name: 'Notes' })),
      row('notes.item', (p) => p.getByRole('listbox', { name: 'Notes' }).getByRole('option'), 1),
      control('notes.title', (p) => p.getByRole('textbox', { name: 'Note title' })),
      control('notes.pin', (p) => btn(p, 'Pin note')),
      control('notes.delete', (p) => btn(p, 'Delete note')),
      control('notes.color', (p) => p.getByRole('group', { name: 'Note color' }).getByRole('button'), 9),
      control('notes.tag', (p) => p.getByRole('textbox', { name: 'Add tag' })),
      control('notes.format', (p) => p.getByRole('region', { name: 'Notes' }).getByRole('toolbar', { name: 'Formatting' }).getByRole('button'), 10),
      control('notes.body', (p) => p.getByRole('textbox', { name: 'Note body' })),
    ],
  },
  {
    id: 'admin-login',
    tag: '@admin',
    themed: false,
    viewports: ['desktop', 'phone'],
    open: async (page) => {
      await page.goto('/admin');
      await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible();
    },
    probes: [
      container('admin.login-card', (p) => p.getByRole('main').getByRole('form')),
      text('admin.login-title', (p) => p.getByRole('heading', { level: 1 })),
      text('admin.login-help', (p) => p.getByRole('main').getByRole('paragraph').first()),
      control('admin.login-field', (p) => p.getByRole('main').getByRole('textbox'), 2),
      control('admin.login-submit', (p) => p.getByRole('button', { name: /sign in/i })),
    ],
  },
  {
    id: 'admin-users',
    tag: '@admin',
    themed: false,
    viewports: ['desktop'],
    open: async (page) => {
      await adminSignIn(page);
      await adminNavItem(page, 'Users').first().click();
      const main = page.getByRole('main');
      await expect(main.getByRole('heading', { name: 'Users' })).toBeVisible();
      await page.waitForTimeout(500);
      // One provisioned account, created through the form once per stack, so the table renders.
      if ((await main.getByRole('table').count()) === 0) {
        await main.getByRole('textbox', { name: 'Username' }).fill(ADMIN_USER.username);
        await main.getByRole('textbox', { name: 'Domain' }).fill(ADMIN_USER.domain);
        await main.getByRole('button', { name: 'Provision user' }).click();
        await expect(main.getByRole('table')).toBeVisible();
        // Re-enter the section so the form's confirmation text does not differ between runs.
        await adminNavItem(page).first().click();
        await adminNavItem(page, 'Users').first().click();
      }
      await expect(main.getByRole('table')).toBeVisible();
    },
    probes: [
      container('admin.nav', (p) => adminNav(p)),
      control('admin.nav-item', (p) => adminNavItem(p), 13),
      container('admin.main', (p) => p.getByRole('main')),
      text('admin.heading', (p) => p.getByRole('main').getByRole('heading', { level: 2 }).first()),
      container('admin.form', (p) => p.getByRole('main').getByRole('form').first()),
      control('admin.field', (p) => p.getByRole('main').getByRole('form').first().getByRole('textbox'), 2),
      control('admin.number', (p) => p.getByRole('main').getByRole('form').first().getByRole('spinbutton'), 2),
      control('admin.submit', (p) => p.getByRole('main').getByRole('form').first().getByRole('button').last()),
      container('admin.table', (p) => p.getByRole('main').getByRole('table')),
      text('admin.th', (p) => p.getByRole('main').getByRole('columnheader'), 5),
      container('admin.tr', (p) => p.getByRole('main').getByRole('row'), 2),
      control('admin.row-control', (p) => p.getByRole('main').getByRole('table').locator('input, button'), 4),
    ],
  },
  // ── phone ──
  {
    id: 'phone-list',
    tag: '@mail',
    viewports: ['phone'],
    open: signedIn,
    probes: [
      container('bar', (p) => p.locator('.shell__bar')),
      control('bar.drawer', (p) => drawerButton(p)),
      text('bar.title', (p) => p.locator('.shell__bar-title')),
      control('bar.compose', (p) => btn(p.locator('.shell__bar'), 'Compose')),
      ...listProbes,
      control('list.more', (p) => p.getByTestId('msg-more-toggle'), 10),
    ],
    rows: { row: '.list__row', scroller: '.list__scroll' },
  },
  {
    id: 'phone-drawer',
    tag: '@shell',
    viewports: ['phone'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      await drawerButton(page).click();
      await expect(sidebarInbox(page)).toBeVisible();
      // The drawer slides in; wait until it rests at the left edge.
      await expect(async () => {
        const box = await page.locator('aside.sidebar').boundingBox();
        expect(box?.x ?? -1).toBeGreaterThanOrEqual(0);
      }).toPass({ timeout: 3_000 });
    },
    probes: [control('bar.drawer', (p) => drawerButton(p)), ...sidebarProbes],
  },
  {
    id: 'phone-reader',
    tag: '@mail',
    viewports: ['phone'],
    open: async (page, ctx) => {
      await openReader(page, ctx);
      await expect(btn(page.getByRole('region', { name: 'Message' }), /Back/)).toBeFocused();
    },
    probes: readerProbes,
    exclude: ['iframe.reader__frame'],
  },
  {
    id: 'phone-compose',
    tag: '@compose',
    viewports: ['phone'],
    open: async (page, ctx) => {
      await signedIn(page, ctx);
      await btn(page.locator('.shell__bar'), 'Compose').click();
      await expect(page.getByRole('dialog', { name: 'Compose message' })).toBeVisible();
    },
    probes: composeProbes,
  },
];

// ── fixture and page preparation ────────────────────────────────────────────

export const test = csrfTest.extend<RestyleOptions>({
  viewportName: ['desktop', { option: true }],
  theme: ['dark', { option: true }],
});
export { expect };

/** Screens for this project, filtered by `T32_SCREENS` (comma-separated ids) when set. */
export function screensFor(): Screen[] {
  const only = process.env['T32_SCREENS'];
  if (!only) return SCREENS;
  const ids = new Set(only.split(',').map((s) => s.trim()));
  return SCREENS.filter((s) => ids.has(s.id));
}

/**
 * Put the page in the measured state: no account-stored appearance (so the seed
 * below is what the app adopts), the theme seed in `mw.theme.prefs`, the fixed
 * clock, reduced motion (config), no service worker (config). `chosen: true` is
 * read by the t32 parser and ignored by the T32-BASE one.
 */
export async function prepare(page: Page, ctx: ScreenContext): Promise<void> {
  await resetAccountAppearance(ctx.request);
  await page.addInitScript((theme) => {
    try {
      localStorage.setItem('mw.theme.prefs', JSON.stringify({ mode: 'fixed', theme, chosen: true }));
    } catch {
      /* storage unavailable: the theme assertion below reports it */
    }
  }, ctx.theme);
  await page.clock.setFixedTime(FIXED_NOW);
}

/** Wait for fonts, the network and layout to settle, then check the theme took effect. */
export async function settle(page: Page, ctx: ScreenContext, screen: Screen): Promise<void> {
  // T32_SEED_CSS: a stylesheet applied after the screen opens, to show a check failing on a
  // deliberate defect. Served from a same-origin URL because the CSP refuses inline styles.
  const seed = process.env['T32_SEED_CSS'];
  if (seed) {
    await page.route('**/__t32_seed.css', (r) => r.fulfill({ status: 200, contentType: 'text/css', body: seed }));
    await page.addStyleTag({ url: new URL('__t32_seed.css', page.url()).href });
  }
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForLoadState('networkidle', { timeout: 5_000 }).catch(() => undefined);
  await page.evaluate(
    () => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
  );
  await page.waitForTimeout(400);
  if (screen.themed !== false) await expect(page.locator('html')).toHaveAttribute('data-theme', ctx.theme);
}

/** The screen's label in output paths: <viewport>/<theme>/<screen>. */
export function outPath(kind: string, ctx: ScreenContext, screen: Screen, ext: string): string {
  const out = process.env['T32_OUT']!;
  const label = process.env['T32_LABEL']!;
  return `${out}/${kind}/${label}/${ctx.viewportName}/${ctx.theme}/${screen.id}.${ext}`;
}
