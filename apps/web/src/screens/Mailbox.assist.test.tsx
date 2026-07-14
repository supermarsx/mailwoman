// V7 (e14b) integration: the Assist chat panel + semantic-search toggle are wired
// into the mailbox, gated on the Assist gateway. A Disabled gateway renders NOTHING
// (the mailbox is unchanged); an enabled one surfaces both.

import { describe, it, expect, beforeEach } from 'vitest';
import { screen } from '@solidjs/testing-library';
import { MailboxScreen } from './Mailbox.tsx';
import { renderWithApp } from '../components/appHarness.tsx';
import { AssistService } from '../modules/assist/index.ts';

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

function enabledAssist(): AssistService {
  return new AssistService(async (input: string) => {
    if (input.includes('/api/assist/config')) {
      return json({
        availability: 'enabled',
        capabilities: ['assistant', 'search-semantic'],
        endpoint_host: 'assist.local',
        include_e2ee: false,
        include_attachments: false,
      });
    }
    return json({});
  });
}

describe('Mailbox Assist wiring', () => {
  beforeEach(() => localStorage.clear());

  it('surfaces the Assist chat panel and semantic-search toggle when enabled', async () => {
    const { app } = renderWithApp(() => <MailboxScreen />, { deps: { assistService: enabledAssist() } });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await app.assist.loadConfig();

    expect(await screen.findByRole('region', { name: 'Assist' })).toBeInTheDocument();
    expect(screen.getByText('Semantic search')).toBeInTheDocument();
  });

  it('renders no Assist affordances when the gateway is disabled', async () => {
    const { app } = renderWithApp(() => <MailboxScreen />);
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

    expect(screen.queryByRole('region', { name: 'Assist' })).toBeNull();
    expect(screen.queryByText('Semantic search')).toBeNull();
  });
});
