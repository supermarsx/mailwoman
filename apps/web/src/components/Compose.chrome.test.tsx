import { ccbccToggle } from './compose/composeHeader.css.ts';
// The composer's chrome: every notice in the bottom status bar, and the
// collapsible header (summary line, remembered state, opens for a problem).

import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { Compose } from './Compose.tsx';
import { clearSettings, loadSettings } from '../api/settings.ts';
import type { Client } from '../api/client.ts';
import type { JmapRequest, JmapResponse, JmapSession } from '../api/jmap-types.ts';
import { chooseComposeFormat } from './compose/nativeHtmlTestHarness.ts';

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: {},
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

function client(): Client {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => ({
    methodResponses: body.methodCalls.map((call) => [call[0], call[0] === 'Identity/get' ? { accountId: 'acct1', state: 's', list: [], notFound: [] } : {}, call[2]] as JmapResponse['methodResponses'][number]),
    sessionState: 's',
  }));
  return {
    login: vi.fn(async () => ({ username: 'me@example.org', accountId: 'acct1' })),
    logout: vi.fn(async () => undefined),
    me: vi.fn(async () => ({ username: 'me@example.org', accountId: 'acct1' })),
    session: vi.fn(async () => SESSION),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  };
}

const entry = (value: unknown, locked = false) => ({ value, source: 'user', locked });
async function configure(settings: Record<string, unknown>): Promise<void> {
  await loadSettings({
    refresh: true,
    fetcher: async (_input, init) => {
      if ((init?.method ?? 'GET') === 'PUT') return new Response(JSON.stringify(entry('x')), { status: 200, headers: { 'content-type': 'application/json' } });
      return new Response(JSON.stringify({ settings }), { status: 200, headers: { 'content-type': 'application/json' } });
    },
  });
}
const POLICIES = (overrides: Record<string, unknown> = {}) => ({
  'mail.compose.checkInvalidRecipients': entry('warn'),
  'mail.compose.checkMissingAttachment': entry('warn'),
  'mail.compose.checkEmptySubject': entry('warn'),
  'mail.externalRecipientWarning': entry('off'),
  ...overrides,
});

async function mount() {
  const app = createAppState(client());
  render(() => <AppContext.Provider value={app}><Compose onClose={() => undefined} /></AppContext.Provider>);
  await chooseComposeFormat('plain');
  return { app, bar: () => screen.getByTestId('compose-status') };
}
const typeBody = (text: string) => fireEvent.input(screen.getByLabelText('Body'), { target: { value: text } });

afterEach(() => { clearSettings(); localStorage.clear(); });

describe('Compose status bar', () => {
  it('is the last row of the composer, always present, and the only live status region for notices', async () => {
    await configure(POLICIES());
    const { bar } = await mount();
    const form = document.querySelector('form.compose')!;
    expect(bar().parentElement).toBe(form);
    expect(form.lastElementChild).toBe(bar());
    expect(bar()).toHaveAttribute('role', 'status');
    // No stacked banner above the fields any more.
    expect(document.querySelector('.compose-alerts, .compose-external, .compose__from-hint')).toBeNull();
  });

  it('does not nag about recipients or a subject on an empty fresh draft', async () => {
    await configure(POLICIES());
    const { bar } = await mount();
    for (const id of ['no-recipients', 'subject', 'invalid', 'attachment']) expect(within(bar()).queryByTestId(`compose-status-${id}`)).toBeNull();
  });

  it('shows no recipients and no subject once the user has typed, as warnings that jump to the field', async () => {
    await configure(POLICIES());
    const { bar } = await mount();
    const node = bar();
    typeBody('Hello there');
    const noRecipients = await within(bar()).findByTestId('compose-status-no-recipients');
    expect(noRecipients).toHaveAttribute('data-severity', 'warning');
    expect(noRecipients).toHaveTextContent('No recipients');
    expect(bar()).toBe(node);
    fireEvent.click(within(noRecipients).getByRole('button', { name: /No recipients/ }));
    expect(screen.getByRole('combobox', { name: 'To' })).toHaveFocus();
    fireEvent.click(within(bar()).getByRole('button', { name: 'No subject' }));
    expect(screen.getByLabelText('Subject')).toHaveFocus();
  });

  it('turns a deny policy into an error item', async () => {
    await configure(POLICIES({ 'mail.compose.checkEmptySubject': entry('deny') }));
    const { bar } = await mount();
    typeBody('Hello');
    expect((await within(bar()).findByRole('button', { name: 'No subject' })).closest('[data-status-id]')).toHaveAttribute('data-severity', 'danger');
  });

  it('flags a mentioned but missing attachment live, in the bar rather than only at send', async () => {
    await configure(POLICIES());
    const { bar } = await mount();
    typeBody('Please see the attached invoice');
    const item = await within(bar()).findByTestId('compose-status-attachment');
    expect(item).toHaveTextContent('No attachment');
    expect(item).toHaveTextContent('mentions an attachment');
  });

  it('removes the tooltip from the To field but keeps its accessible name', async () => {
    await configure(POLICIES());
    await mount();
    const to = screen.getByRole('combobox', { name: 'To' });
    expect(to).toHaveAttribute('aria-label', 'To');
    expect(to).toHaveAttribute('data-tooltip', 'off');
    expect(to).not.toHaveAttribute('title');
  });
});

describe('Compose header collapse', () => {
  async function fill() {
    fireEvent.input(screen.getByRole('combobox', { name: 'To' }), { target: { value: 'Ana <ana@example.org>, b@example.org, c@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Re: Invoice' } });
  }

  it('is expanded by default and folds to a one-line summary that is remembered', async () => {
    await configure(POLICIES());
    await mount();
    await fill();
    const toggle = screen.getByTestId('compose-header-toggle');
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(screen.getByTestId('compose-header-summary')).toHaveTextContent('To: Ana +2 · Subject: Re: Invoice');
    expect(localStorage.getItem('mw.compose.headerOpen')).toBe('0');
  });

  it('starts folded when the setting says collapsed, once there is something to summarise', async () => {
    await configure(POLICIES({ 'mail.compose.headerCollapse': entry('collapsed') }));
    await mount();
    expect(screen.getByTestId('compose-header-toggle')).toHaveAttribute('aria-expanded', 'true');
    await fill();
    await waitFor(() => expect(screen.getByTestId('compose-header-toggle')).toHaveAttribute('aria-expanded', 'false'));
  });

  it('opens for a problem clicked in the status bar and for focus inside a field', async () => {
    await configure(POLICIES({ 'mail.compose.headerCollapse': entry('collapsed') }));
    const { bar } = await mount();
    fireEvent.input(screen.getByRole('combobox', { name: 'To' }), { target: { value: 'ana@example.org' } });
    typeBody('Hello');
    const toggle = screen.getByTestId('compose-header-toggle');
    await waitFor(() => expect(toggle).toHaveAttribute('aria-expanded', 'false'));
    fireEvent.click(await within(bar()).findByRole('button', { name: 'No subject' }));
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByLabelText('Subject')).toHaveFocus();
  });

  it('keeps the Cc and Bcc toggle in the To row', async () => {
    await configure(POLICIES());
    await mount();
    const row = screen.getByRole('combobox', { name: 'To' }).closest('.compose__to')!;
    const toggle = within(row as HTMLElement).getByTestId('compose-head-ccbcc');
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    expect(toggle).toHaveClass(ccbccToggle);
    expect(toggle).toHaveAttribute('aria-controls');
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByRole('combobox', { name: 'Cc' })).toBeInTheDocument();
  });
});
