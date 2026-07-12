import { test, expect } from '@playwright/test';
import { engineLogin, injectViaSmtp, messageRow, sidebarInbox } from './helpers.ts';

/**
 * V2 realtime push (plan §1.3, §2.2), end-to-end against the engine stack.
 * With the app open + idle on the Inbox, a brand-new delivery injected straight
 * over SMTP (the app takes no action) must appear in the list WITHOUT a manual
 * refresh: engine watch-loop resync -> broadcast StateChange over /jmap/ws ->
 * the push client reacts -> refetch -> the row renders. startRealtime() is wired
 * post-login (App.tsx effect on app.me()).
 */

// retries: the engine's Greenmail IMAP watch connection can transiently break
// under accumulated session load (resync "Broken pipe"), delaying ingestion; a
// fresh retry (new login -> new connection) recovers. Escalated as an engine
// robustness issue.
test.describe.configure({ mode: 'serial', retries: 2 });

test.describe('V2 realtime push (engine mode)', () => {
  test('a new delivery appears without a manual refresh', async ({ page }) => {
    test.slow(); // waits on the engine watch interval + push delivery.
    await engineLogin(page);
    // engineLogin selects the Inbox; stay idle on it (do NOT click Inbox again).
    await expect(sidebarInbox(page)).toBeVisible();

    const subject = `Push ${Date.now()}`;
    await injectViaSmtp({
      from: 'Push Bot <pushbot@example.org>',
      subject,
      text: `realtime delivery ${subject}`,
    });

    // No refresh, no navigation: the row must appear purely via the push path.
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 90_000 });
  });
});
