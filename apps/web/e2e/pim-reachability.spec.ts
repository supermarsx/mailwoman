import { test, expect } from '@playwright/test';
import { engineLogin, gotoModule } from './pim-helpers.ts';

/**
 * V3 reachability gate (plan §3 e12, DoD "mounted + wired"): the four PIM modules
 * must each boot REACHABLE from the app-shell nav rail and be ENGINE-BACKED — the
 * explicit mount step V2 lacked. This spec logs into the engine stack once and
 * clicks through every nav entry, asserting each module's own root renders (its
 * lazy chunk resolves + the engine-backed slice mounts). The per-module specs
 * then prove each feature end-to-end (create → render → reload → still there).
 */

test.describe('PIM modules are reachable from the shell nav (engine mode)', () => {
  test('Calendar / Tasks / Notes / Contacts each boot from the nav rail', async ({ page }) => {
    await engineLogin(page);

    // Each nav-rail entry is present (the frozen AppModule registry drives them).
    for (const id of ['calendar', 'tasks', 'notes', 'contacts'] as const) {
      await expect(page.getByTestId(`nav-${id}`)).toBeVisible();
    }

    // Clicking each navigates the hash router and mounts the real module.
    await gotoModule(page, 'calendar');
    await expect(page.getByRole('button', { name: '+ Event' })).toBeVisible();

    await gotoModule(page, 'tasks');
    await expect(page.getByLabel('New task title')).toBeVisible();

    await gotoModule(page, 'notes');
    await expect(page.getByRole('button', { name: '+ New note' })).toBeVisible();

    await gotoModule(page, 'contacts');
    await expect(page.getByRole('button', { name: 'New contact' })).toBeVisible();

    // Mail is still reachable (no regression to the existing shell surface).
    await page.getByRole('navigation', { name: 'Mailboxes' }).getByRole('button', { name: 'Inbox' }).click();
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
  });
});
