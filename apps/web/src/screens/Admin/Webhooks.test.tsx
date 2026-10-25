// Admin › Integrations › Webhooks (t28-e18, worker f34): the list, creating an
// outbound webhook (a refused URL is shown at the URL field), creating an inbound
// one (the secret and the signing scheme shown once), delete behind a
// confirmation with If-Match, the 412 reload, the unavailable state, axe-clean and
// without inline styles.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { Webhooks } from './Webhooks.tsx';
import type { AdminWebhook } from '../../api/admin/webhooks.ts';
import { expectNoAxeViolations, expectNoInlineStyle } from './shell/uxTestkit.ts';
import { t } from '../../test/i18n.ts';

const OUT: AdminWebhook = {
  id: 'wh_1',
  direction: 'outbound',
  accountId: 'alice@example.org',
  label: 'Ticketing',
  url: 'https://hooks.example/new-mail',
  events: ['message.received'],
  createdAt: '2026-10-25T10:00:00Z',
  createdBy: 'admin:root',
};
const IN: AdminWebhook = {
  id: 'whin_1',
  direction: 'inbound',
  accountId: 'bob@example.org',
  label: '',
  actions: ['run_rules'],
  createdAt: '2026-10-25T09:00:00Z',
  createdBy: 'admin:root',
  lastUsedAt: null,
};

function json(status: number, body: unknown, etag?: string): Response {
  const headers: Record<string, string> = { 'content-type': 'application/json' };
  if (etag) headers['ETag'] = etag;
  return new Response(status === 204 ? null : JSON.stringify(body), { status, headers });
}

interface Call {
  url: string;
  method: string;
  body: unknown;
  ifMatch: string | null;
}

function stubFetch(handler: (c: Call) => Response): Call[] {
  const calls: Call[] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: string, init?: RequestInit) => {
      const headers = new Headers(init?.headers);
      const c: Call = {
        url: String(input),
        method: init?.method ?? 'GET',
        body: init?.body ? JSON.parse(String(init.body)) : undefined,
        ifMatch: headers.get('If-Match'),
      };
      calls.push(c);
      return handler(c);
    }),
  );
  return calls;
}

const listing = (webhooks: AdminWebhook[], etag = '"e1"') =>
  json(200, { webhooks, eventKinds: ['message.received', 'message.flags'], actionKinds: ['run_rules', 'sieve'], etag }, etag);

describe('Admin › Webhooks', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('lists outbound and inbound webhooks without inline styles or axe violations', async () => {
    stubFetch(() => listing([OUT, IN]));
    const { container } = render(() => <Webhooks />);
    const table = await screen.findByTestId('admin-webhooks-table');
    expect(within(table).getByText('Ticketing')).toBeInTheDocument();
    expect(within(table).getByText('https://hooks.example/new-mail')).toBeInTheDocument();
    expect(within(table).getByText(t('admin-webhooks-event-message-received'))).toBeInTheDocument();
    expect(within(table).getByText(t('admin-webhooks-no-label'))).toBeInTheDocument();
    expect(within(table).getByText(t('admin-webhooks-action-run-rules'))).toBeInTheDocument();
    expect(within(table).getByText(t('admin-webhooks-never-used'))).toBeInTheDocument();
    expectNoInlineStyle(container);
    await expectNoAxeViolations(container);
  });

  it('checks the form before any request, then shows a refused URL at the URL field', async () => {
    const calls = stubFetch((c) =>
      c.method === 'POST'
        ? json(422, { code: 'webhookUrlRefused', detail: 'The URL host is private.', field: '/url' })
        : listing([]),
    );
    render(() => <Webhooks />);
    await screen.findByText(t('admin-webhooks-empty'));

    fireEvent.click(screen.getByTestId('admin-webhooks-create'));
    expect(await screen.findByText(t('admin-webhooks-error-account'))).toBeInTheDocument();
    expect(screen.getByText(t('admin-webhooks-error-url'))).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'POST')).toHaveLength(0);

    fireEvent.input(screen.getByLabelText(t('admin-webhooks-add-account')), { target: { value: 'alice@example.org' } });
    fireEvent.input(screen.getByLabelText(t('admin-webhooks-add-url')), { target: { value: 'http://10.0.0.5/hook' } });
    fireEvent.click(screen.getByTestId('admin-webhooks-create'));
    const url = screen.getByLabelText(t('admin-webhooks-add-url'));
    await waitFor(() => expect(url).toHaveAttribute('aria-invalid', 'true'));
    expect(screen.getByText('The URL host is private.')).toBeInTheDocument();
    const post = calls.find((c) => c.method === 'POST');
    expect(post?.ifMatch).toBe('"e1"');
    expect(post?.body).toEqual({
      direction: 'outbound',
      accountId: 'alice@example.org',
      label: '',
      url: 'http://10.0.0.5/hook',
      events: ['message.received'],
    });
  });

  it('creates an inbound webhook and shows its secret and how to sign, once', async () => {
    let made = false;
    const calls = stubFetch((c) => {
      if (c.method === 'POST') {
        made = true;
        return json(201, { webhook: { ...IN, id: 'whin_9', label: 'CRM' }, secret: 'ab'.repeat(32), etag: '"e2"' }, '"e2"');
      }
      return made ? listing([{ ...IN, id: 'whin_9', label: 'CRM' }], '"e2"') : listing([]);
    });
    render(() => <Webhooks />);
    await screen.findByText(t('admin-webhooks-empty'));
    fireEvent.click(screen.getByLabelText(new RegExp(t('admin-webhooks-add-inbound-help'))));
    fireEvent.input(screen.getByLabelText(t('admin-webhooks-add-account')), { target: { value: 'bob@example.org' } });
    fireEvent.input(screen.getByLabelText(t('admin-webhooks-add-label')), { target: { value: 'CRM' } });
    fireEvent.click(screen.getByLabelText(t('admin-webhooks-action-sieve')));
    fireEvent.click(screen.getByTestId('admin-webhooks-create'));

    const secret = await screen.findByTestId('admin-webhooks-secret-value');
    expect(secret.textContent).toBe('ab'.repeat(32));
    expect(screen.getByTestId('admin-webhooks-secret').textContent).toContain('whin_9');
    expect(calls.find((c) => c.method === 'POST')?.body).toMatchObject({
      direction: 'inbound',
      accountId: 'bob@example.org',
      label: 'CRM',
      actions: ['run_rules', 'sieve'],
    });
    fireEvent.click(screen.getByRole('button', { name: t('admin-webhooks-secret-done') }));
    await waitFor(() => expect(screen.queryByTestId('admin-webhooks-secret')).toBeNull());
  });

  it('deletes after confirmation with the list tag, and reloads on a stale tag', async () => {
    let deleted = false;
    let stale = true;
    const calls = stubFetch((c) => {
      if (c.method === 'DELETE') {
        if (stale) {
          stale = false;
          return json(412, { code: 'stale', updatedBy: 'admin:ops', updatedAt: '2026-10-25T11:00:00Z' });
        }
        deleted = true;
        return json(204, null, '"e3"');
      }
      return deleted ? listing([IN], '"e3"') : listing([OUT, IN]);
    });
    render(() => <Webhooks />);
    fireEvent.click(await screen.findByRole('button', { name: t('admin-webhooks-delete-named', { name: 'Ticketing' }) }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog.textContent).toContain('Ticketing');
    fireEvent.click(within(dialog).getByRole('button', { name: t('admin-webhooks-delete') }));
    expect(await screen.findByText(t('admin-webhooks-error-stale'))).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'DELETE')[0]?.ifMatch).toBe('"e1"');

    fireEvent.click(await screen.findByRole('button', { name: t('admin-webhooks-delete-named', { name: 'Ticketing' }) }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: t('admin-webhooks-delete') }));
    expect(await screen.findByText(t('admin-webhooks-deleted'))).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByText('Ticketing')).toBeNull());
    expect(calls.filter((c) => c.method === 'DELETE')[1]?.url).toMatch(/\/webhooks\/wh_1$/);
  });

  it('says so when the server has no webhook management', async () => {
    stubFetch(() => new Response('not found', { status: 404 }));
    render(() => <Webhooks />);
    expect(await screen.findByText(t('admin-webhooks-unavailable'))).toBeInTheDocument();
  });
});
