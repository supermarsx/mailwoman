import { test, expect, type Page } from '@playwright/test';
import { engineLogin, injectViaSmtp, waitForInboxMessage, messageRow } from './helpers.ts';

/**
 * V2 search operators (plan §0.1, §2.1), end-to-end. The mounted SearchBox sends
 * the query as the JMAP Email/query `filter.text`; the engine routes it through
 * mw-search's operator parser (from:/subject:/has:attachment/…) and the matching
 * ids replace the message list. Seed two messages — different senders, one with
 * an attachment — and assert each operator narrows correctly.
 */

// retries: seeding waits on the engine ingesting injected mail; a fresh retry
// recovers if the watch loop transiently stalls.
test.describe.configure({ mode: 'serial', retries: 2 });

async function search(page: Page, query: string): Promise<void> {
  const box = page.getByRole('searchbox', { name: 'Search mail' });
  await box.fill(query);
  await box.press('Enter');
}

async function clearSearch(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Clear' }).click();
}

test.describe('V2 search operators (engine mode)', () => {
  test('from: / subject: / has:attachment narrow the result set', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    const stamp = Date.now();
    const alpha = `alphaword${stamp}`; // unique token in Alice's subject
    const beta = `betaword${stamp}`; //  unique token in Bob's subject

    await injectViaSmtp({ from: 'Alice Adams <alice@example.org>', subject: `report ${alpha}`, text: 'plain, no file' });
    await injectViaSmtp({
      from: 'Bob Brown <bob@example.org>',
      subject: `report ${beta}`,
      text: 'has a file',
      withAttachment: { filename: 'notes.txt', content: 'quarterly numbers' },
    });
    await waitForInboxMessage(page, alpha, 120_000);
    await waitForInboxMessage(page, beta, 120_000);

    // subject: narrows to the message whose subject carries the token.
    await search(page, `subject:${alpha}`);
    await expect(messageRow(page, alpha)).toBeVisible();
    await expect(messageRow(page, beta)).toHaveCount(0);
    await clearSearch(page);
    await expect(messageRow(page, beta)).toBeVisible();

    // from: narrows to the sender (Alice, no attachment).
    await search(page, 'from:alice');
    await expect(messageRow(page, alpha)).toBeVisible();
    await expect(messageRow(page, beta)).toHaveCount(0);
    await clearSearch(page);
    await expect(messageRow(page, alpha)).toBeVisible();

    // has:attachment narrows to the message carrying a file (Bob's).
    await search(page, 'has:attachment');
    await expect(messageRow(page, beta)).toBeVisible();
    await expect(messageRow(page, alpha)).toHaveCount(0);
    await clearSearch(page);

    // A subject token matching neither message -> empty results.
    await expect(messageRow(page, alpha)).toBeVisible();
    await search(page, `subject:nomatch${stamp}`);
    await expect(page.getByText('No messages')).toBeVisible();
  });
});
