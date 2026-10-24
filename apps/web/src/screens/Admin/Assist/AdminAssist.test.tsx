import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor, cleanup } from '@solidjs/testing-library';
import { AdminAssist } from './index.tsx';
import { AdminAssistApi, parseAdminAssistConfig, type Fetcher } from './service.ts';

afterEach(() => cleanup());

// The bodies below are the server's, not this client's idea of them. They mirror
// `crates/mw-server/src/v7_mount.rs`:
//   - the config object: `assist_admin_wire`, which GET returns and which
//     `AssistAdminReq` requires back, whole, on PUT;
//   - the status object: `assist_status`, returned by GET /admin/assist/status,
//     PUT /admin/assist and POST /admin/assist/kill;
//   - the 422 for a local program `MW_ASSIST_LOCAL_PROGRAMS` does not list:
//     `local_program_refusal`;
//   - the kill body: `AssistKillReq`, `{ on }`.
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

const RUNNING = {
  enabled: true,
  running: true,
  endpointHost: 'llm.example.test',
  restartPending: false,
  stoppedReason: null,
  localPrograms: [] as string[],
};

interface Call {
  method: string;
  path: string;
  body: unknown;
}

/** A fetch that serves the admin Assist routes from `state` and records every call. */
function server(initial: { config: unknown; status: unknown; getStatus?: number; localPrograms?: string[] }): {
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
      const adapter = (body as { adapter: { kind: string; program?: string } | null }).adapter;
      if (adapter?.kind === 'local-process' && !(initial.localPrograms ?? []).includes(adapter.program ?? '')) {
        return json({ error: 'adapter.program is not listed in MW_ASSIST_LOCAL_PROGRAMS' }, 422);
      }
      state.config = body;
      state.status = { ...RUNNING, restartPending: true, localPrograms: initial.localPrograms ?? [] };
      return json(state.status);
    }
    if (input === '/admin/assist/kill' && method === 'POST') {
      const on = (body as { on: boolean }).on;
      state.config = { ...(state.config as object), enabled: !on };
      state.status = on
        ? { ...RUNNING, enabled: false, running: false, endpointHost: null }
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
    expect(screen.getByLabelText('AI assistant chat')).not.toBeChecked();
    expect(screen.getByLabelText('Account ids whose mail may be sent, one per line')).toHaveValue('acct-1\nacct-2');
    expect(screen.getByLabelText('Allow attachments to be sent')).toBeChecked();
    expect(screen.getByLabelText('Allow end-to-end-encrypted content to be sent')).not.toBeChecked();
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'The AI assistant is running. Requests go to llm.example.test.',
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

    fireEvent.click(screen.getByLabelText('AI assistant chat'));
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
          'Could not load the AI assistant configuration. Saving is disabled so the stored configuration is not overwritten.',
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
    await waitFor(() => expect(screen.getByRole('button', { name: 'Stop the AI assistant now' })).toBeInTheDocument());
    expect(screen.queryByRole('button', { name: 'Turn the AI assistant on' })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Stop the AI assistant now' }));
    await waitFor(() => expect(writes()).toHaveLength(1));
    expect(writes()[0]).toEqual({ method: 'POST', path: '/admin/assist/kill', body: { on: true } });
    await waitFor(() =>
      expect(screen.getByTestId('assist-status')).toHaveTextContent('The AI assistant is off. Every AI assistant request is refused.'),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Turn the AI assistant on' }));
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
      status: { ...RUNNING, running: false, endpointHost: null },
    });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'The AI assistant is on but not running because no usable endpoint is configured.',
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

describe('admin assist screen: local program', () => {
  const PROGRAMS = ['/opt/llm/run', '/opt/llm/other'];
  const NO_ENDPOINT = { ...RUNNING, running: false, endpointHost: null };

  it('does not offer a local program while the operator lists none, and says how to enable one', async () => {
    const { api } = server({ config: { ...STORED, adapter: null }, status: NO_ENDPOINT });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());
    const kinds = Array.from(screen.getByLabelText('Endpoint type').querySelectorAll('option')).map((o) => o.value);
    expect(kinds).toEqual(['none', 'open-ai-compatible', 'anthropic']);
    expect(screen.getByTestId('assist-local-program-enable')).toHaveTextContent('MW_ASSIST_LOCAL_PROGRAMS');
  });

  it('offers the listed programs only, with arguments as a list, and saves the choice', async () => {
    const { api, writes } = server({
      config: { ...STORED, adapter: null },
      status: { ...NO_ENDPOINT, localPrograms: PROGRAMS },
      localPrograms: PROGRAMS,
    });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());
    expect(screen.queryByTestId('assist-local-program-enable')).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText('Endpoint type'), { target: { value: 'local-process' } });
    const program = screen.getByLabelText('Program');
    expect(Array.from(program.querySelectorAll('option')).map((o) => o.value)).toEqual(PROGRAMS);
    expect(program).toHaveValue('/opt/llm/run');
    expect(screen.getByText(/starts this program for each AI assistant request/)).toBeInTheDocument();

    fireEvent.change(program, { target: { value: '/opt/llm/other' } });
    fireEvent.change(screen.getByLabelText('Arguments, one per line'), {
      target: { value: '--model\n\n small \n' },
    });
    fireEvent.click(saveButton());
    await waitFor(() => expect(writes()).toHaveLength(1));
    expect((writes()[0]?.body as { adapter: unknown }).adapter).toEqual({
      kind: 'local-process',
      program: '/opt/llm/other',
      args: ['--model', 'small'],
    });
  });

  it('shows why a stored program that is no longer listed is not running, and that saving it is refused', async () => {
    const stored = { ...STORED, adapter: { kind: 'local-process', program: '/usr/bin/old', args: [] } };
    const { api, writes } = server({
      config: stored,
      status: { ...NO_ENDPOINT, stoppedReason: 'local-program-not-allowed', localPrograms: PROGRAMS },
      localPrograms: PROGRAMS,
    });
    render(() => <AdminAssist api={api} />);
    await waitFor(() => expect(saveButton()).toBeEnabled());
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'The AI assistant is not running. The configured program /usr/bin/old is not listed in MW_ASSIST_LOCAL_PROGRAMS on the server, so the server did not start it.',
    );
    expect(screen.getByLabelText('Program')).toHaveValue('/usr/bin/old');
    expect(screen.getByTestId('assist-local-program-unlisted')).toBeInTheDocument();

    fireEvent.click(saveButton());
    await waitFor(() =>
      expect(screen.getByRole('alert')).toHaveTextContent(
        'The configuration was not saved because the server does not allow this program.',
      ),
    );
    expect(writes()).toHaveLength(1);
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
