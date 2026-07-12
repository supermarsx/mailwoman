import { test, expect, type Page } from '@playwright/test';
import { engineLogin, injectViaSmtp, messageRow, waitForInboxMessage } from './helpers.ts';
import { readFile } from 'node:fs/promises';

/**
 * V2 export (plan §8 / DoD). Two paths:
 *  - the mounted Reader "Export" toolbar button -> a `.eml` browser download of
 *    the whole message (fetched over the JMAP blob downloadUrl, message/rfc822);
 *  - the server export endpoint GET /api/export/{id}?format=eml|mbox (engine
 *    mode, e14) -> asserts the raw-EML and mbox byte shapes directly.
 */

const CAP = ['urn:ietf:params:jmap:core', 'urn:ietf:params:jmap:mail'];

async function jmap(
  page: Page,
  method: string,
  args: Record<string, unknown>,
  callId: string,
): Promise<Record<string, unknown>> {
  const res = await page.request.post('/jmap/api', {
    data: { using: CAP, methodCalls: [[method, args, callId]] },
  });
  const body = await res.json();
  return body.methodResponses.find((m: unknown[]) => m[2] === callId)![1];
}

/** The first Email id in the Inbox (an unfiltered Email/query returns none — the
 *  engine needs an inMailbox filter). */
async function firstInboxEmailId(page: Page): Promise<string> {
  const session = await (await page.request.get('/jmap/session')).json();
  const accountId: string = session.primaryAccounts['urn:ietf:params:jmap:mail'];
  const boxes = (await jmap(page, 'Mailbox/get', { accountId }, 'c0')).list as { id: string; role: string }[];
  const inbox = boxes.find((b) => b.role === 'inbox')!.id;
  const q = await jmap(page, 'Email/query', { accountId, filter: { inMailbox: inbox }, limit: 1 }, 'q');
  return (q.ids as string[])[0];
}

test.describe.configure({ mode: 'serial', retries: 2 });

test.describe('V2 export (engine mode)', () => {
  test('Reader "Export" downloads the message as .eml', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    const subject = `Export ${Date.now()}`;
    await injectViaSmtp({ from: 'Exporter <exp@example.org>', subject, text: `export body ${subject}` });
    await waitForInboxMessage(page, subject, 90_000);

    await messageRow(page, subject).first().click();
    await expect(page.getByRole('heading', { name: subject })).toBeVisible();

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.getByTestId('reader-export').click(),
    ]);
    expect(download.suggestedFilename()).toMatch(/\.eml$/i);

    // The downloaded bytes are the real RFC822 message (correct content).
    const path = await download.path();
    const content = await readFile(path, 'utf8');
    expect(content).toContain(`Subject: ${subject}`);
    expect(content).toContain(`export body ${subject}`);
  });

  test('export endpoint serves EML and mbox with correct bytes', async ({ page }) => {
    await engineLogin(page);
    const id = await firstInboxEmailId(page);

    // EML: raw RFC822 (has header lines).
    const eml = await page.request.get(`/api/export/${id}?format=eml`);
    expect(eml.ok()).toBeTruthy();
    const emlText = await eml.text();
    expect(emlText).toMatch(/^(From|Subject|Date|To|Received|MIME-Version):/im);

    // mbox: the same message wrapped with a `From ` separator line at the top.
    const mbox = await page.request.get(`/api/export/${id}?format=mbox`);
    expect(mbox.ok()).toBeTruthy();
    expect(await mbox.text()).toMatch(/^From /);
  });
});
