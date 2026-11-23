// V7 (e14b) integration: the Assist chat panel + semantic-search toggle are wired
// into the mailbox, gated on the Assist gateway. A Disabled gateway renders NOTHING
// (the mailbox is unchanged); an enabled one surfaces both.

import { describe, it, expect, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { MailboxScreen } from './Mailbox.tsx';
import { renderWithApp, mkEmail } from '../components/appHarness.tsx';
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
    await app.login({ username: 'me@example.org', password: 'p' });
    await app.assist.loadConfig();

    expect(await screen.findByRole('region', { name: 'AI assistant' })).toBeInTheDocument();
    expect(screen.getByText('Semantic search')).toBeInTheDocument();
  });

  it('renders no Assist affordances when the gateway is disabled', async () => {
    const { app } = renderWithApp(() => <MailboxScreen />);
    await app.login({ username: 'me@example.org', password: 'p' });

    expect(screen.queryByRole('region', { name: 'AI assistant' })).toBeNull();
    expect(screen.queryByText('Semantic search')).toBeNull();
  });
});

describe('Mailbox Assist chat context', () => {
  beforeEach(() => localStorage.clear());

  it('sends the open message as plain subject + preview labelled with its mailbox id', async () => {
    const sent: Array<{ input: { context: unknown[] } }> = [];
    const service = new AssistService(async (input: string, init?: RequestInit) => {
      if (input.includes('/api/assist/config')) {
        return json({ availability: 'enabled', capabilities: ['assistant'], endpoint_host: 'assist.local', include_e2ee: false, include_attachments: false });
      }
      if (input.includes('/api/assist/invoke')) {
        sent.push(JSON.parse(String(init?.body ?? '{}')) as (typeof sent)[number]);
        return json({ text: 'ok', disclosure: { endpoint_host: 'assist.local', sent: [], withheld: [] } });
      }
      return json({});
    });
    const { app } = renderWithApp(() => <MailboxScreen />, { deps: { assistService: service }, emails: [mkEmail('m1')] });
    await app.login({ username: 'me@example.org', password: 'p' });
    await app.assist.loadConfig();
    await app.openMessage('m1');
    const input = await screen.findByRole('textbox', { name: 'Message the AI assistant' });
    fireEvent.input(input, { target: { value: 'Summarise this' } });
    fireEvent.submit(input.closest('form')!);
    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]!.input.context).toEqual([
      { account: 'acct1', folder: 'inbox', text: 'Subject m1\npreview m1', kind: 'plain' },
    ]);
  });
});
