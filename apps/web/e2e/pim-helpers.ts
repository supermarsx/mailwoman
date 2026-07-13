import { expect, type Page } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * Shared helpers for the V3 PIM live E2E (plan §3 e12). The four PIM modules
 * (Calendar / Tasks / Notes / Contacts) are mounted into the SAME app shell the
 * V1/V2 specs drive, reachable from the "Apps" nav rail (`data-testid="nav-<id>"`)
 * and hash-routed (`#/calendar`, `#/tasks`, `#/notes`, `#/contacts`). They run
 * against the engine-mode server (:8090) over its auto-seeded Mailwoman-native
 * default collections (a default calendar + task list + address book are seeded
 * on first get, per e8) — so these specs create real PIM objects through the
 * real UI WITHOUT configuring a CalDAV account (the CalDAV round-trip itself is
 * proven at the Rust level by e11's `caldav-carddav-conformance` job).
 *
 * Login is the same engine-mode login the V1 `imap-engine` spec uses (the server
 * dials Greenmail; the browser only ever talks to :8090). `engineLogin` is
 * re-exported so the PIM specs import a single module.
 */
export { engineLogin } from './helpers.ts';

/** A short unique token so assertions are immune to leftover/seeded PIM objects. */
export function uid(): string {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 6)}`;
}

/**
 * Navigate to a PIM module via its nav-rail entry and wait until the module's own
 * root (`[data-module="<id>"]`, rendered inside the shell's `main.module-pane`)
 * is visible — the lazy chunk resolves on first activation, so we await the
 * module root rather than assuming it is synchronous.
 */
export async function gotoModule(
  page: Page,
  id: 'calendar' | 'tasks' | 'notes' | 'contacts',
): Promise<void> {
  await page.getByTestId(`nav-${id}`).click();
  await expect(page.locator(`main.module-pane[data-surface="${id}"]`)).toBeVisible();
  await expect(page.locator(`[data-module="${id}"]`)).toBeVisible();
}

/**
 * Reload the page and land back on the mailbox shell. The engine-mode session
 * cookie survives a reload (the app re-checks `/api/me` on boot and renders the
 * mailbox directly), so this proves data read after the reload came from the
 * ENGINE, not from in-memory client state. Re-authenticates defensively if the
 * login form appears (it should not).
 */
export async function reloadToShell(page: Page): Promise<void> {
  await page.reload();
  const compose = page.getByRole('button', { name: 'Compose' });
  const signIn = page.getByRole('button', { name: 'Sign in' });
  await expect(compose.or(signIn)).toBeVisible();
  if (await signIn.isVisible().catch(() => false)) {
    await engineLogin(page);
  }
  await expect(compose).toBeVisible();
}
