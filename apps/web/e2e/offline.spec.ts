import { test, expect } from '@playwright/test';
import { engineLogin, composeSelf, sidebarInbox, messageRow } from './helpers.ts';

/**
 * V2 offline queue + replay (plan §1.2, §2.5), end-to-end. Compose while offline
 * -> the mutation is captured in the IndexedDB outbound queue instead of sent;
 * on reconnect the queue drains FIFO and the send actually dispatches.
 *
 * Runtime note: app.online() is request-failure-driven (client.onNetwork), NOT
 * navigator.onLine — so we force a real request to fail first (click Inbox)
 * before it flips to offline. The browser 'online' event on reconnect is what
 * triggers the replay.
 */

test.describe.configure({ mode: 'serial' });

test.describe('V2 offline queue + replay (engine mode)', () => {
  test('compose while offline queues, then replays and sends on reconnect', async ({ page, context }) => {
    test.slow();
    await engineLogin(page);

    // Go offline, then force a request to fail so app.online() flips to false.
    await context.setOffline(true);
    await sidebarInbox(page).click();
    await expect(page.locator('.sidebar__offline')).toBeVisible({ timeout: 15_000 });

    // Composing now takes the offline path: queued, not sent.
    const subject = `Offline ${Date.now()}`;
    await composeSelf(page, subject, `queued while offline ${subject}`);
    await expect(page.getByText('Queued — will send when back online')).toBeVisible();

    // Reconnect. The idle app won't issue a request on its own, so click Inbox:
    // that first successful request fires onNetwork(up) -> drains the queue, and
    // its replayed send self-delivers back to the Inbox. The DELIVERED message
    // (durable) is the proof the queued send actually dispatched — stronger than
    // the transient replay toast.
    await context.setOffline(false);
    await expect(async () => {
      await sidebarInbox(page).click();
      await expect(messageRow(page, subject).first()).toBeVisible({ timeout: 3_000 });
    }).toPass({ timeout: 45_000 });
  });
});
