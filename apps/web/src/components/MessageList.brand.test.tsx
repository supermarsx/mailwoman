import { describe, it, expect, beforeEach } from 'vitest';
import { cleanup, screen } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';
import { clearSettings } from '../api/settings.ts';
import type { ListAuthSummary } from '../api/jmap-types.ts';

// A list row shows a brand logo only when the message's authentication summary
// proves it; a spoofed From on a brand domain gets the initials.
const pass: ListAuthSummary = { dkim: { result: 'pass', domain: 'github.com' }, dmarc: { result: 'pass', aligned: true } };
const from = [{ name: 'GitHub', email: 'notifications@github.com' }];

async function rowIcon(auth: ListAuthSummary | null | undefined): Promise<string | null> {
  const { app, result } = renderWithApp(() => <MessageList />, { emails: [mkEmail('a', { subject: 'Alpha', from, ...(auth === undefined ? {} : { mailwomanAuth: auth }) })] });
  await app.login({ username: 'me@example.org', password: 'p' });
  await screen.findByText('Alpha');
  const icon = result.container.querySelector<HTMLElement>('[data-sender-icon]');
  const name = icon?.getAttribute('data-sender-icon') ?? null;
  cleanup();
  return name;
}

describe('MessageList brand logos', () => {
  beforeEach(() => { localStorage.clear(); clearSettings(); });
  it('shows the logo for an authenticated message', async () => {
    expect(await rowIcon(pass)).toBe('github');
  });
  it('shows the initials for a failed check', async () => {
    expect(await rowIcon({ ...pass, dmarc: { result: 'fail', aligned: false } })).toBeNull();
    expect(await rowIcon({ ...pass, dkim: { result: 'pass', domain: 'evil.example' } })).toBeNull();
  });
  it('shows the initials when the message has no verdict', async () => {
    expect(await rowIcon(null)).toBeNull();
    expect(await rowIcon(undefined)).toBeNull();
  });
});
