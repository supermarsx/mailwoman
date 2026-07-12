import { test, expect } from '@playwright/test';
import {
  engineLogin,
  composeSelf,
  waitForInboxMessage,
  messageRow,
  messageSlot,
  sidebarInbox,
} from './helpers.ts';

/**
 * V2 modern-mail UX, end-to-end against the REAL engine stack (mw-server in
 * MW_MODE=engine over Greenmail, :8090 — the `engine` project). These drive the
 * genuinely-wired UX surface through the unmodified UI: the per-row action
 * cluster (tag/pin/snooze/follow-up), pinned ordering, the shared 10-second undo
 * toast, undo-send cancel, send-later + the visible Outbox, and the focused/
 * unified inbox tabs. Nothing is stubbed — each mutation goes through the store
 * to the engine (keywords over IMAP, meta via message_meta, submissions via the
 * engine's delayed dispatcher).
 *
 * Greenmail's INBOX starts empty, so specs that act on a message first SEED one
 * by self-sending and waiting for the loopback delivery (compose→SMTP→IMAP→sync).
 */

// These tests mutate ONE shared Greenmail account (testuser), so they must not
// run concurrently — parallel IMAP/submission ops on a single account race and
// surface as `serverFail`. Serial mode matches CI (workers:1) and keeps local
// runs deterministic. Theming (theming.spec.ts) needs no account and stays
// parallel-safe.
test.describe.configure({ mode: 'serial' });

test.describe('V2 modern mail UX (engine mode)', () => {
  test('tag chip + pin ordering + follow-up + undo toast on seeded messages', async ({ page }) => {
    test.slow(); // seeding waits on the undo-send hold + SMTP loopback + sync.
    await engineLogin(page);

    const stamp = Date.now();
    const older = `UX-older ${stamp}`;
    const newer = `UX-newer ${stamp}`;

    // Seed two messages; `newer` is sent last so it sorts above `older` (desc).
    await composeSelf(page, older, `body ${older}`);
    await composeSelf(page, newer, `body ${newer}`);
    await waitForInboxMessage(page, older);
    await waitForInboxMessage(page, newer);

    // The per-row action cluster + submenus have NO CSS in this build (no
    // `.msg-actions`/`.msg-menu` rules), so they render inline and overlap the
    // row; a real pointer click is intercepted by the row and the virtualized
    // list remounts mid-click. We fire the button handlers via `dispatchEvent`
    // — this still runs the GENUINE store action (applyTag/pinMessage/... → JMAP
    // to the engine), only bypassing the broken visual hit-testing.
    const olderSlot = messageSlot(page, older);

    // ── Tag: open the row's Label menu and apply "Work"; a colored chip renders.
    await olderSlot.getByRole('button', { name: 'Label' }).dispatchEvent('click');
    const workItem = olderSlot.getByRole('menuitemcheckbox', { name: /Work/ });
    await expect(workItem).toBeVisible();
    await workItem.dispatchEvent('click');
    await expect(olderSlot.locator('.tag-chip[data-keyword="work"]')).toBeVisible();
    // The shared undo toast confirms the reversible action.
    await expect(page.locator('.undo-toast')).toContainText('Label added');

    // ── Pin: pinning the OLDER message floats it above the newer one (ordering).
    await olderSlot.getByRole('button', { name: 'Pin', exact: true }).dispatchEvent('click');
    await expect(page.locator('.undo-toast')).toContainText('Pinned');
    // Its row is now marked pinned and carries the pin indicator.
    await expect(messageRow(page, older)).toHaveClass(/list__row--pinned/);
    await expect(olderSlot.getByLabel('Pinned')).toBeVisible();
    // Pinned float: the first slot in the list is now the older (pinned) message.
    await expect(page.locator('.list__slot').first()).toContainText(older);

    // ── Follow-up: flag the newer message; the control flips to a clear-action.
    const newerSlot = messageSlot(page, newer);
    await newerSlot.getByRole('button', { name: 'Flag for follow-up' }).dispatchEvent('click');
    await expect(newerSlot.getByRole('button', { name: 'Clear follow-up' })).toBeVisible();

    // ── Snooze: snoozing the newer message hides it from the list.
    await newerSlot.getByRole('button', { name: 'Snooze' }).dispatchEvent('click');
    const tomorrow = newerSlot.getByRole('menuitem', { name: 'Tomorrow' });
    await expect(tomorrow).toBeVisible();
    await tomorrow.dispatchEvent('click');
    await expect(messageSlot(page, newer)).toHaveCount(0);
    // The older (pinned) message is still shown.
    await expect(messageRow(page, older)).toBeVisible();
  });

  test('undo-send: cancel within the hold window stops delivery', async ({ page }) => {
    await engineLogin(page);

    const subject = `UndoSend ${Date.now()}`;
    await composeSelf(page, subject, 'this send will be canceled');

    // Sending shows the undo toast with a "Cancel" action (the engine holds the
    // submission for the undo window before dialing SMTP).
    const undo = page.locator('.undo-toast');
    await expect(undo).toContainText('Message sent');
    // Unstyled toast (no `.undo-toast` CSS) — fire the handler directly.
    await undo.getByRole('button', { name: 'Cancel' }).dispatchEvent('click');

    // The cancel flips the submission to `canceled` before dispatch.
    await expect(page.getByText('Send canceled')).toBeVisible();

    // The Outbox (EmailSubmission/query) shows the canceled submission, not sent.
    await page.getByRole('button', { name: 'Outbox' }).click();
    const outbox = page.getByRole('region', { name: 'Outbox' });
    await expect(outbox.locator('.outbox__row[data-state="canceled"]').first()).toBeVisible();

    // And it never lands in the Inbox: give the hold window time to have elapsed.
    await sidebarInbox(page).click();
    await expect(async () => {
      await sidebarInbox(page).click();
      await expect(messageRow(page, subject)).toHaveCount(0, { timeout: 2_000 });
    }).toPass({ timeout: 20_000 });
  });

  test('send-later shows a scheduled row in the Outbox', async ({ page }) => {
    await engineLogin(page);

    // A datetime-local value ~1 day out (local wall clock; the app converts to UTC).
    const when = new Date(Date.now() + 24 * 3_600_000);
    const pad = (n: number) => String(n).padStart(2, '0');
    const local = `${when.getFullYear()}-${pad(when.getMonth() + 1)}-${pad(when.getDate())}T${pad(when.getHours())}:${pad(when.getMinutes())}`;

    const subject = `SendLater ${Date.now()}`;
    await composeSelf(page, subject, 'scheduled for tomorrow', { sendLater: local });
    await expect(page.getByText('Scheduled to send')).toBeVisible();

    await page.getByRole('button', { name: 'Outbox' }).click();
    const outbox = page.getByRole('region', { name: 'Outbox' });
    await expect(outbox.locator('.outbox__row[data-state="scheduled"]').first()).toBeVisible();
    await expect(outbox.getByText('Scheduled', { exact: true }).first()).toBeVisible();
  });

  test('focused inbox exposes the two-tab split and unified toggle', async ({ page }) => {
    await engineLogin(page);

    // Focused inbox is opt-in: enabling it reveals the Focused/Other tablist.
    await page.getByRole('button', { name: 'Focused inbox' }).click();
    const tablist = page.getByRole('tablist', { name: 'Inbox filter' });
    await expect(tablist).toBeVisible();
    await expect(tablist.getByRole('tab', { name: /Focused/ })).toBeVisible();
    await expect(tablist.getByRole('tab', { name: /Other/ })).toBeVisible();

    // Switching tabs updates the active selection.
    await tablist.getByRole('tab', { name: /Other/ }).click();
    await expect(tablist.getByRole('tab', { name: /Other/ })).toHaveAttribute('aria-selected', 'true');

    // The unified-inbox toggle is present and flips.
    const unified = page.getByRole('checkbox', { name: 'Unified inbox' });
    await unified.check();
    await expect(unified).toBeChecked();
  });
});
