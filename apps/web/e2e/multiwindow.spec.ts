import { test, expect } from '@playwright/test';
import { engineLogin, injectViaSmtp, messageSlot, sidebarInbox, waitForInboxMessage } from './helpers.ts';

/**
 * V2 multi-window consistency (plan §1.4, §2.6). e13 wired the cross-tab store
 * sync over a BroadcastChannel named `mw-store` (the SharedWorker proxy is a
 * documented follow-up). Every successful mutation in one tab broadcasts a
 * refetch frame; peer tabs re-load the current mailbox. So a pin applied in
 * tab A must appear in tab B WITHOUT any interaction in B.
 *
 * We assert on PIN (engine-local `message_meta`, reliably returned on a fresh
 * refetch) rather than a label — Greenmail here exposes only INBOX (no folder to
 * move to) and its custom-keyword round-trip is not dependable, so a label set
 * in A wouldn't survive B's server refetch.
 */

// retries: seeding depends on the engine ingesting an injected delivery, which
// can stall when its Greenmail IMAP connection transiently breaks under load; a
// fresh retry recovers. (Engine robustness issue, escalated.)
test.describe.configure({ mode: 'serial', retries: 2 });

test.describe('V2 multi-window (BroadcastChannel)', () => {
  test('a pin applied in one tab appears in another without acting on it', async ({ context }) => {
    test.slow();
    const tabA = await context.newPage();
    await engineLogin(tabA);

    const subject = `MW ${Date.now()}`;
    await injectViaSmtp({ from: 'MW Bot <mwbot@example.org>', subject, text: `multiwindow ${subject}` });

    // Wait for tab A to see it first (confirms the engine ingested it into the
    // store), so tab B — opened next — finds it immediately. Generous timeout
    // absorbs a slow watch-loop resync under load.
    await waitForInboxMessage(tabA, subject, 90_000);

    // Second tab shares the session cookie -> lands straight in the mailbox.
    const tabB = await context.newPage();
    await tabB.goto('/');
    await expect(sidebarInbox(tabB)).toBeVisible();
    await waitForInboxMessage(tabB, subject, 45_000);

    // Pin the message in tab A (unstyled action cluster -> dispatchEvent). This
    // persists to message_meta and broadcasts a refetch over the mw-store channel.
    const slotA = messageSlot(tabA, subject).first();
    await slotA.getByRole('button', { name: 'Pin', exact: true }).dispatchEvent('click');
    await expect(slotA.getByLabel('Pinned')).toBeVisible();

    // Tab B reflects it WITHOUT any interaction in tab B — cross-window sync:
    // the pin indicator appears on B's row after the broadcast-driven refetch.
    await expect(messageSlot(tabB, subject).first().getByLabel('Pinned')).toBeVisible({ timeout: 15_000 });
  });
});
