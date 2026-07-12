import { test, expect, type Page } from '@playwright/test';
import { engineLogin, injectViaSmtp, waitForInboxMessage, messageRow } from './helpers.ts';

/**
 * V2 search, end-to-end. The mounted SearchBox sends the query as the JMAP
 * Email/query `filter.text`; the engine routes it to mw-search and the matching
 * ids replace the message list. We seed two messages with distinctive tokens and
 * assert the search narrows to exactly the matching message and clears back.
 *
 * SCOPE NOTE: field operators (from:/subject:/has:attachment) do NOT work
 * end-to-end in this build — the engine wraps the whole `filter.text` as a
 * literal all-fields term (mw-engine search_index.rs build_search_query) instead
 * of running it through mw_search::parse_query, so `subject:foo` searches for the
 * literal words "subject" AND "foo". Full-text narrowing (the path exercised
 * here) works; the operator gap is escalated to the crate owners.
 */

// retries: seeding waits on the engine ingesting injected mail, which can stall
// when its Greenmail IMAP connection transiently breaks under load; a fresh
// retry recovers. (Engine robustness issue, escalated.)
test.describe.configure({ mode: 'serial', retries: 2 });

async function search(page: Page, query: string): Promise<void> {
  const box = page.getByRole('searchbox', { name: 'Search mail' });
  await box.fill(query);
  await box.press('Enter');
}

async function clearSearch(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Clear' }).click();
}

test.describe('V2 search (engine mode)', () => {
  test('full-text search narrows the message list to matches', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    const stamp = Date.now();
    const alpha = `alphaword${stamp}`; // unique token, only in msg1's subject
    const beta = `betaword${stamp}`; //   unique token, only in msg2's subject

    await injectViaSmtp({ from: 'Alice <alice@example.org>', subject: `report ${alpha}`, text: 'one' });
    await injectViaSmtp({ from: 'Bob <bob@example.org>', subject: `report ${beta}`, text: 'two' });
    await waitForInboxMessage(page, alpha, 90_000);
    await waitForInboxMessage(page, beta, 90_000);

    // Search msg1's token -> only msg1 remains in the (results-replaced) list.
    await search(page, alpha);
    await expect(messageRow(page, alpha)).toBeVisible();
    await expect(messageRow(page, beta)).toHaveCount(0);

    // Clearing restores the full list.
    await clearSearch(page);
    await expect(messageRow(page, beta)).toBeVisible();

    // Search msg2's token -> only msg2.
    await search(page, beta);
    await expect(messageRow(page, beta)).toBeVisible();
    await expect(messageRow(page, alpha)).toHaveCount(0);
    // Wait for the clear's full-list refetch to settle before searching again,
    // else it can resolve after (and overwrite) the next search's result.
    await clearSearch(page);
    await expect(messageRow(page, alpha)).toBeVisible();

    // A token matching neither message -> empty results.
    await search(page, `nomatch${stamp}`);
    await expect(page.getByText('No messages')).toBeVisible();
  });
});
