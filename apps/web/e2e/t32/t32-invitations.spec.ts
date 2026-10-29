import { expect, test } from '@playwright/test';
import { ENGINE_CREDS, engineLogin, injectViaSmtp, messageRow, waitForInboxMessage } from '../helpers.ts';

test('received invitations require a click and reject a forged cancellation', async ({ page }) => {
  const tag = `${Date.now()}-${test.info().project.name}`;
  const uid = `invite-${tag}@example.org`;
  const subject = `Invitation ${tag}`;
  const organizer = 'organizer@example.org';
  function calendar(method: string, sequence: number): string {
    return [
      'BEGIN:VCALENDAR', 'VERSION:2.0', 'PRODID:-//Mailwoman//Test//EN', `METHOD:${method}`,
      'BEGIN:VEVENT', `UID:${uid}`, `SEQUENCE:${sequence}`, 'DTSTAMP:20261025T120000Z',
      'DTSTART:20261026T120000Z', 'DURATION:PT1H', `SUMMARY:Planning ${tag}`,
      `ORGANIZER:mailto:${organizer}`, `ATTENDEE:mailto:${ENGINE_CREDS.selfAddress}`,
      'END:VEVENT', 'END:VCALENDAR', '',
    ].join('\r\n');
  }
  await injectViaSmtp({
    from: organizer, subject, text: 'Calendar invitation',
    attachments: [{ filename: 'invite.ics', contentType: 'text/calendar', content: calendar('REQUEST', 1) }],
  });
  await engineLogin(page);
  await waitForInboxMessage(page, subject);
  const applies: boolean[] = [];
  page.on('request', (request) => {
    if (!request.url().endsWith('/jmap/api') || request.method() !== 'POST') return;
    const body = request.postDataJSON() as { methodCalls?: [string, { apply?: boolean }, string][] };
    for (const [method, args] of body.methodCalls ?? []) {
      if (method === 'CalendarEvent/itip') applies.push(args.apply === true);
    }
  });
  await messageRow(page, subject).click();
  const card = page.getByRole('region', { name: 'Calendar invitation' });
  await expect(card.getByText(`Planning ${tag}`)).toBeVisible();
  expect(applies).toEqual([false]);
  // t53: the invitation lists this account's address, so the card answers it.
  // Accept without sending a reply: the event is stored, nothing is mailed.
  await card.getByRole('checkbox', { name: /Send a response to/ }).uncheck();
  await card.getByRole('button', { name: 'Accept' }).click();
  await expect(card.getByText('Accepted. No response was sent.')).toBeVisible();
  expect(applies.filter(Boolean)).toEqual([]);

  await page.reload();
  await waitForInboxMessage(page, subject);
  await messageRow(page, subject).click();
  await expect(card.locator('[data-card="badge"]')).toHaveText('Accepted');
  await expect(card.getByRole('button', { name: 'Add to calendar' })).toHaveCount(0);

  const forged = `Forged cancellation ${tag}`;
  await injectViaSmtp({
    from: 'outsider@example.org', subject: forged, text: 'Forged cancellation',
    attachments: [{ filename: 'cancel.ics', contentType: 'text/calendar', content: calendar('CANCEL', 2) }],
  });
  await waitForInboxMessage(page, forged);
  await messageRow(page, forged).click();
  await expect(card.getByText('The sender is not the stored organizer. Nothing will be changed.')).toBeVisible();
  await expect(card.getByRole('button')).toHaveCount(0);
  expect(applies.filter(Boolean)).toHaveLength(0);
});
