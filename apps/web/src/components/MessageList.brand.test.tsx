import { describe, it, expect, beforeEach } from 'vitest';
import { cleanup, screen } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import { clearSettings } from '../api/settings.ts';
import type { ListAuthSummary } from '../api/jmap-types.ts';

// A list row shows the brand logo for the From domain and its subdomains, also for a message
// with no cached verdict; only a verdict that says the check failed gives the initials.
const pass: ListAuthSummary = { dkim: { result: 'pass', domain: 'github.com' }, dmarc: { result: 'pass', aligned: true } };
let address = 'notifications@github.com';

async function rowIcon(auth: ListAuthSummary | null | undefined): Promise<string | null> {
  const { app, result } = renderWithApp(() => <MessageList />, { emails: [mkEmail('a', { subject: 'Alpha', from: [{ name: 'GitHub', email: address }], ...(auth === undefined ? {} : { mailwomanAuth: auth }) })] });
  await app.login({ username: 'me@example.org', password: 'p' });
  await screen.findByText('Alpha');
  const icon = result.container.querySelector<HTMLElement>('[data-sender-icon]');
  const name = icon?.getAttribute('data-sender-icon') ?? null;
  cleanup();
  return name;
}

describe('MessageList brand logos', () => {
  beforeEach(() => { localStorage.clear(); clearSettings(); address = 'notifications@github.com'; });
  it('shows the logo for an authenticated message', async () => {
    expect(await rowIcon(pass)).toBe('github');
  });
  it('shows the initials for a failed check', async () => {
    expect(await rowIcon({ ...pass, dmarc: { result: 'fail', aligned: false } })).toBeNull();
    expect(await rowIcon({ dkim: { result: 'fail', domain: 'github.com' }, dmarc: { result: 'none', aligned: false } })).toBeNull();
  });
  it('shows the logo when the message has no verdict yet', async () => {
    expect(await rowIcon(null)).toBe('github');
    expect(await rowIcon(undefined)).toBe('github');
  });
  it('matches subdomain senders, registry brands and rejects lookalikes', async () => {
    for (const [sender, id] of [['a@e.amazon.com', 'amazon'], ['a@mail.zalando.com', 'zalando'], ['a@em.paypal.com', 'paypal'], ['a@news.vinted.com', 'vinted'], ['a@service.fnac.pt', 'fnac'], ['a@mail.ryanair.com', 'ryanair']] as const) {
      address = sender;
      expect(await rowIcon(undefined), sender).toBe(id);
    }
    for (const sender of ['a@paypal.com.evil.io', 'a@evilpaypal.com', 'a@gmail.com']) { address = sender; expect(await rowIcon(undefined), sender).toBeNull(); }
  });
});
