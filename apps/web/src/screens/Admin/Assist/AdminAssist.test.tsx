import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@solidjs/testing-library';
import { AdminAssist } from './index.tsx';
import { AdminAssistApi, parseAdminAssistConfig, type Fetcher } from './service.ts';

afterEach(() => cleanup());

// The bodies below are the server's, not this client's idea of them. They mirror
// `crates/mw-server/src/v7_mount.rs`:
//   - the config object: `assist_admin_wire` (line 2441), which GET returns and
//     which `AssistAdminReq` (line 2455) requires back, whole, on PUT;
//   - the status object: `assist_status` (line 2532), returned by
//     GET /admin/assist/status, PUT /admin/assist and POST /admin/assist/kill;
//   - the kill body: `AssistKillReq` (line 2688), `{ on }`.
// The server-side round trip of the same objects is
// `crates/mw-server/tests/t28_assist_admin.rs`. This is a unit test of the screen
// against a mocked fetch; it does not show the screen is wired to a server.

const STORED = {
  enabled: true,
  adapter: {
    kind: 'open-ai-compatible',
    baseUrl: 'https://llm.example.test/v1',
    apiKey: 'sk-t28',
    chatModel: 'chat-t28',
    embedModel: 'embed-t28',
    sttModel: 'stt-t28',
  },
  capabilityGrants: ['summarize', 'dictation'],
  dataCeilings: {
    accounts: ['acct-1', 'acct-2'],
    folders: ['inbox'],
    includeE2ee: false,
    includeAttachments: true,
  },
};

const RUNNING = { enabled: true, running: true, endpointHost: 'llm.example.test', restartPending: false };

interface Call {
  method: string;
  path: string;
  body: unknown;
}

/** A fetch that serves the admin Assist routes from `state` and records every call. */
function server(initial: { config: unknown; status: unknown; getStatus?: number }): {
  api: AdminAssistApi;
  calls: Call[];
  writes: () => Call[];
} {
  const state = { config: initial.config, status: initial.status };
  const calls: Call[] = [];
  const json = (body: unknown, status = 200): Response =>
    new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
  const fetcher: Fetcher = vi.fn(async (input: string, init?: RequestInit) => {
    const method = init?.method ?? 'GET';
    const body = typeof init?.body === 'string' ? (JSON.parse(init.body) as unknown) : undefined;
    calls.push({ method, path: input, body });
    if (input === '/admin/assist' && method === 'GET') return json(state.config, initial.getStatus ?? 200);
    if (input === '/admin/assist/status') return json(state.status);
    if (input === '/admin/assist' && method === 'PUT') {
      state.config = body;
      state.status = { ...RUNNING, restartPending: true };
      return json(state.status);
    }
    if (input === '/admin/assist/kill' && method === 'POST') {
      const on = (body as { on: boolean }).on;
      state.config = { ...(state.config as object), enabled: !on };
      state.status = on
        ? { enabled: false, running: false, endpointHost: null, restartPending: false }
        : RUNNING;
      return json(state.status);
    }
    return json({ error: 'not found' }, 404);
  });
  return { api: new AdminAssistApi(fetcher), calls, writes: () => calls.filter((c) => c.method !== 'GET') };
}

const saveButton = (): HTMLElement => screen.getByRole('button', { name: 'Save configuration' });

describe('admin assist screen', () => {
  it('shows the stored configuration and what the server reports as running', async () => {
    const { api } = server({ config: STORED, status: RUNNING });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());

    expect(screen.getByLabelText('Base URL')).toHaveValue('https://llm.example.test/v1');
    expect(screen.getByLabelText('Chat model')).toHaveValue('chat-t28');
    expect(screen.getByLabelText('Summarize')).toBeChecked();
    expect(screen.getByLabelText('Assistant chat')).not.toBeChecked();
    expect(screen.getByLabelText('Account ids whose mail may be sent, one per line')).toHaveValue('acct-1\nacct-2');
    expect(screen.getByLabelText('Allow attachments to be sent')).toBeChecked();
    expect(screen.getByLabelText('Allow end-to-end-encrypted content to be sent')).not.toBeChecked();
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'Assist is running. Requests go to llm.example.test.',
    );
  });

  it('saving an untouched form sends back exactly what was loaded', async () => {
    const { api, writes } = server({ config: STORED, status: RUNNING });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());

    fireEvent.click(saveButton());
    await waitFor(() => expect(writes()).toHaveLength(1));
    expect(writes()[0]).toEqual({ method: 'PUT', path: '/admin/assist', body: STORED });
  });

  it('sends edits in the same shape and says when they wait for a restart', async () => {
    const { api, writes } = server({ config: STORED, status: RUNNING });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());

    fireEvent.click(screen.getByLabelText('Assistant chat'));
    fireEvent.click(screen.getByLabelText('Summarize'));
    fireEvent.click(screen.getByLabelText('Allow end-to-end-encrypted content to be sent'));
    fireEvent.change(screen.getByLabelText('Folder ids to limit sending to, one per line'), {
      target: { value: 'inbox\n\n archive \n' },
    });
    fireEvent.input(screen.getByLabelText('Chat model'), { target: { value: '' } });
    fireEvent.click(saveButton());

    await waitFor(() => expect(writes()).toHaveLength(1));
    expect(writes()[0]?.body).toEqual({
      enabled: true,
      adapter: {
        kind: 'open-ai-compatible',
        baseUrl: 'https://llm.example.test/v1',
        apiKey: 'sk-t28',
        // chatModel was emptied: omitted, so the server applies its default.
        embedModel: 'embed-t28',
        sttModel: 'stt-t28',
      },
      capabilityGrants: ['dictation', 'assistant'],
      dataCeilings: {
        accounts: ['acct-1', 'acct-2'],
        folders: ['inbox', 'archive'],
        includeE2ee: true,
        includeAttachments: true,
      },
    });
    await waitFor(() =>
      expect(screen.getByTestId('assist-notice')).toHaveTextContent(
        'Saved. The change takes effect when the server restarts.',
      ),
    );
  });

  it('does not offer Save when the configuration could not be read', async () => {
    // The pre-26.20 server answered an unconfigured deployment with only `enabled`.
    for (const broken of [
      { config: { enabled: false }, status: RUNNING },
      { config: STORED, status: RUNNING, getStatus: 500 },
      { config: { ...STORED, dataCeilings: { include_e2ee: false, include_attachments: false } }, status: RUNNING },
    ]) {
      const { api, writes } = server(broken);
      render(() => <AdminAssist api={api} />);
      await waitFor(() =>
        expect(screen.getByRole('alert')).toHaveTextContent(
          'Could not load the Assist configuration. Saving is disabled so the stored configuration is not overwritten.',
        ),
      );
      expect(saveButton()).toBeDisabled();
      fireEvent.click(saveButton());
      expect(writes()).toEqual([]);
      cleanup();
    }
  });

  it('stops Assist through the kill route and reports the answer', async () => {
    const { api, writes } = server({ config: STORED, status: RUNNING });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(screen.getByRole('button', { name: 'Stop Assist now' })).toBeInTheDocument());
    expect(screen.queryByRole('button', { name: 'Turn Assist on' })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Stop Assist now' }));
    await waitFor(() => expect(writes()).toHaveLength(1));
    expect(writes()[0]).toEqual({ method: 'POST', path: '/admin/assist/kill', body: { on: true } });
    await waitFor(() =>
      expect(screen.getByTestId('assist-status')).toHaveTextContent('Assist is off. Every Assist request is refused.'),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Turn Assist on' }));
    await waitFor(() => expect(writes()).toHaveLength(2));
    expect(writes()[1]).toEqual({ method: 'POST', path: '/admin/assist/kill', body: { on: false } });

    // While that request is in flight, Save is not offered: it would send the
    // on/off state from before it.
    expect(saveButton()).toBeDisabled();
    await waitFor(() => expect(saveButton()).toBeEnabled());
    // A later Save carries the state the kill route left, not the one first loaded.
    fireEvent.click(saveButton());
    await waitFor(() => expect(writes()).toHaveLength(3));
    expect((writes()[2]?.body as { enabled: boolean }).enabled).toBe(true);
  });

  it('refuses to save an OpenAI-compatible endpoint with no base URL', async () => {
    const { api, writes } = server({
      config: { ...STORED, adapter: null },
      status: { enabled: true, running: false, endpointHost: null, restartPending: false },
    });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'Assist is on but not running because no usable endpoint is configured.',
    );

    fireEvent.change(screen.getByLabelText('Endpoint type'), { target: { value: 'open-ai-compatible' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Enter the endpoint'));
    expect(writes()).toEqual([]);

    fireEvent.input(screen.getByLabelText('Base URL'), { target: { value: 'http://127.0.0.1:8199/v1' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(writes()).toHaveLength(1));
    expect((writes()[0]?.body as { adapter: unknown }).adapter).toEqual({
      kind: 'open-ai-compatible',
      baseUrl: 'http://127.0.0.1:8199/v1',
    });
  });
});

describe('parseAdminAssistConfig', () => {
  it('accepts the server shape and nothing that only resembles it', () => {
    expect(parseAdminAssistConfig(STORED)).toEqual(STORED);
    expect(parseAdminAssistConfig({ ...STORED, adapter: null }).adapter).toBeNull();
    for (const bad of [
      null,
      { enabled: false },
      { ...STORED, capabilityGrants: ['send'] },
      { ...STORED, adapter: { OpenAiCompatible: { base_url: 'http://mock' } } },
      { ...STORED, enabled: 'yes' },
    ]) {
      expect(() => parseAdminAssistConfig(bad)).toThrow();
    }
  });
});
