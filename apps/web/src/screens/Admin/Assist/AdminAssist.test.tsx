import { describe, it, expect, vi, afterEach } from 'vitest';
import { fireEvent, screen, waitFor, cleanup, within } from '@solidjs/testing-library';
import { AdminAssist, assistConfig, assistValues, type RateLimitApi } from './index.tsx';
import { AdminAssistApi, parseAdminAssistConfig, type AdminAssistConfig, type Fetcher } from './service.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle, problem } from '../shell/uxTestkit.ts';

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
    // GET never returns the key, only whether one is stored.
    hasApiKey: true,
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
  const json = (body: unknown, status = 200): Response => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
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
      // As the server does: the key is stored, and reads back only as `hasApiKey`.
      const sent = body as { adapter: Record<string, unknown> | null };
      if (sent.adapter !== null && sent.adapter['kind'] !== 'local-process') {
        const { apiKey, ...rest } = sent.adapter;
        const prev = (state.config as { adapter: { hasApiKey?: boolean } | null }).adapter;
        const has = typeof apiKey === 'string' ? apiKey !== '' : prev?.hasApiKey === true;
        state.config = { ...sent, adapter: { ...rest, hasApiKey: has } };
      } else {
        state.config = body;
      }
      state.status = { ...RUNNING, restartPending: true, localPrograms: initial.localPrograms ?? [] };
      return json(state.status);
    }
    if (input === '/admin/assist/kill' && method === 'POST') {
      const on = (body as { on: boolean }).on;
      state.config = { ...(state.config as object), enabled: !on };
      state.status = on ? { ...RUNNING, enabled: false, running: false, endpointHost: null } : RUNNING;
      return json(state.status);
    }
    return json({ error: 'not found' }, 404);
  });
  return { api: new AdminAssistApi(fetcher), calls, writes: () => calls.filter((c) => c.method !== 'GET') };
}

const NO_RATE: RateLimitApi = { load: async () => ({ kind: 'unavailable' }), save: vi.fn() };

function rateSetting(over: Partial<AdminSetting> = {}): AdminSetting {
  return {
    key: 'assistant.rateLimitPerMin',
    family: false,
    kind: { type: 'int', min: 0, max: 10000 },
    default: 0,
    nullable: false,
    scopes: 'global',
    userOverride: false,
    category: 'assistant',
    mapsTo: 'v7_mount.rs',
    env: { var: 'MW_ASSIST_RATE_LIMIT_PER_MIN', precedence: 'pin' },
    dedicatedRoute: null,
    scopeId: 'account',
    hot: true,
    ceiling: false,
    userVisible: false,
    globalValue: 30,
    allowUserOverride: false,
    globalLocked: false,
    updatedBy: 'root',
    updatedAt: 1_761_300_000,
    effective: { value: 30, source: 'global' },
    envState: { var: 'MW_ASSIST_RATE_LIMIT_PER_MIN', precedence: 'pin', set: false, value: null },
    overrides: {},
    etag: '"r1"',
    ...over,
  };
}

function setup(s: ReturnType<typeof server>, rate: RateLimitApi = NO_RATE) {
  return renderInShell(() => <AdminAssist api={s.api} rateLimit={rate} />);
}

const saveButton = (): HTMLElement => screen.getByRole('button', { name: 'Save configuration' });
const ready = async (): Promise<void> => {
  await waitFor(() => expect(screen.getByTestId('save-bar')).toBeInTheDocument());
};

describe('admin assist screen', () => {
  it('shows the stored configuration and what the server reports as running', async () => {
    const s = server({ config: STORED, status: RUNNING });
    setup(s);
    await ready();
    expect(screen.getByRole('textbox', { name: 'Base URL' })).toHaveValue('https://llm.example.test/v1');
    expect(screen.getByRole('textbox', { name: 'Chat model' })).toHaveValue('chat-t28');
    expect(screen.getByRole('checkbox', { name: 'Summarize' })).toBeChecked();
    expect(screen.getByRole('checkbox', { name: 'AI assistant chat' })).not.toBeChecked();
    expect(within(screen.getByRole('list', { name: 'Account ids whose mail may be sent' })).getAllByRole('listitem').map((li) => li.textContent?.trim())).toEqual(['acct-1', 'acct-2']);
    expect(screen.getByRole('checkbox', { name: 'Allow attachments to be sent' })).toBeChecked();
    expect(screen.getByTestId('assist-status')).toHaveTextContent('The AI assistant is running. Requests go to llm.example.test.');
    // Endpoint fields take effect on restart.
    expect(within(screen.getByTestId('setting-baseUrl')).getByText('Applies after the server restarts.')).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
  });

  it('sends edits in the server shape: list editors, omitted empty optional fields', async () => {
    const s = server({ config: STORED, status: RUNNING });
    setup(s);
    await ready();
    fireEvent.click(screen.getByRole('checkbox', { name: 'AI assistant chat' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Summarize' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'Allow end-to-end-encrypted content to be sent' }));
    const folders = screen.getByRole('textbox', { name: 'Folder ids to limit sending to' });
    fireEvent.input(folders, { target: { value: ' archive ' } });
    fireEvent.keyDown(folders, { key: 'Enter' });
    fireEvent.input(screen.getByRole('textbox', { name: 'Chat model' }), { target: { value: '' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect(s.writes()[0]?.body).toEqual({
      enabled: true,
      adapter: { kind: 'open-ai-compatible', baseUrl: 'https://llm.example.test/v1', embedModel: 'embed-t28', sttModel: 'stt-t28' },
      capabilityGrants: ['dictation', 'assistant'],
      dataCeilings: { accounts: ['acct-1', 'acct-2'], folders: ['inbox', 'archive'], includeE2ee: true, includeAttachments: true },
    });
    await waitFor(() => expect(screen.getByTestId('admin-form-save-toast')).toHaveTextContent(/Saved/));
  });

  it('does not offer the form when the configuration could not be read', async () => {
    for (const broken of [
      { config: { enabled: false }, status: RUNNING },
      { config: STORED, status: RUNNING, getStatus: 500 },
    ]) {
      const s = server(broken);
      setup(s);
      await waitFor(() => expect(screen.getByTestId('page-state')).toHaveAttribute('data-state', 'error'));
      expect(screen.queryByRole('button', { name: 'Save configuration' })).toBeNull();
      expect(s.writes()).toEqual([]);
      cleanup();
    }
  });

  it('stopping asks first, goes through the kill route and reports the answer; a later Save carries that state', async () => {
    const s = server({ config: STORED, status: RUNNING });
    setup(s);
    await ready();
    fireEvent.click(screen.getByRole('button', { name: 'Stop the AI assistant now' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(s.writes()).toEqual([]);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Stop the AI assistant now' }));
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect(s.writes()[0]).toEqual({ method: 'POST', path: '/admin/assist/kill', body: { on: true } });
    await waitFor(() => expect(screen.getByTestId('assist-status')).toHaveTextContent('The AI assistant is off. Every AI assistant request is refused.'));
    fireEvent.click(screen.getByRole('button', { name: 'Turn the AI assistant on' }));
    await waitFor(() => expect(s.writes()).toHaveLength(2));
    expect(s.writes()[1]).toEqual({ method: 'POST', path: '/admin/assist/kill', body: { on: false } });
    fireEvent.click(screen.getByRole('checkbox', { name: 'Recap' }));
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(3));
    expect((s.writes()[2]?.body as { enabled: boolean }).enabled).toBe(true);
  });

  it('refuses to save an OpenAI-compatible endpoint with no base URL', async () => {
    const s = server({ config: { ...STORED, adapter: null }, status: { ...RUNNING, running: false, endpointHost: null } });
    setup(s);
    await ready();
    expect(screen.getByTestId('assist-status')).toHaveTextContent('The AI assistant is on but not running because no usable endpoint is configured.');
    fireEvent.change(screen.getByRole('combobox', { name: 'Endpoint type' }), { target: { value: 'open-ai-compatible' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(screen.getByText('Enter the endpoint\'s base URL, starting with http:// or https://.')).toBeInTheDocument());
    expect(s.writes()).toEqual([]);
    fireEvent.input(screen.getByRole('textbox', { name: 'Base URL' }), { target: { value: 'http://127.0.0.1:8199/v1' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect((s.writes()[0]?.body as { adapter: unknown }).adapter).toEqual({ kind: 'open-ai-compatible', baseUrl: 'http://127.0.0.1:8199/v1' });
  });

  it('has no axe violations and no inline style', async () => {
    const { container } = setup(server({ config: STORED, status: RUNNING }));
    await ready();
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('admin assist screen: unreadable stored key', () => {
  it('says the stored key does not open with the server key, and lets a new configuration be saved as it is', async () => {
    const calls: { method: string; path: string }[] = [];
    const fetcher: Fetcher = vi.fn(async (input: string, init?: RequestInit) => {
      const method = init?.method ?? 'GET';
      calls.push({ method, path: input });
      const json = (body: unknown, status = 200): Response => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
      if (method === 'PUT') return json({ ...RUNNING, running: false, endpointHost: null, restartPending: true });
      // As `assist_read_error` answers both reads.
      return json({ error: 'does not open with MW_SERVER_KEY', code: 'config-unreadable' }, 500);
    });
    renderInShell(() => <AdminAssist api={new AdminAssistApi(fetcher)} rateLimit={NO_RATE} />);
    await waitFor(() => expect(screen.getByTestId('assist-unreadable')).toHaveTextContent('MW_SERVER_KEY'));
    expect(saveButton()).toBeEnabled();
    fireEvent.click(saveButton());
    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
  });
});

describe('admin assist screen: API key', () => {
  it('says a key is stored without showing it, and a save that leaves it alone omits it so the server keeps it', async () => {
    const s = server({ config: STORED, status: RUNNING });
    const { container } = setup(s);
    await ready();
    expect(screen.getByTestId('assist-api-key-stored')).toHaveTextContent('A key is stored.');
    expect(screen.getByLabelText('Replace the API key')).toHaveValue('');
    expect(document.querySelector('input[aria-label="API key"]')).toBeNull();
    for (const input of Array.from(container.querySelectorAll('input'))) expect(input.value).not.toContain('sk-');
    // Typed into and emptied again is still "keep the stored key", not "clear it".
    fireEvent.input(screen.getByLabelText('Replace the API key'), { target: { value: 'sk-oops' } });
    fireEvent.input(screen.getByLabelText('Replace the API key'), { target: { value: '' } });
    fireEvent.click(screen.getByRole('checkbox', { name: 'Recap' }));
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect((s.writes()[0]?.body as { adapter: Record<string, unknown> }).adapter).not.toHaveProperty('apiKey');
  });

  it('sends a replacement key, and an empty string to remove the stored one', async () => {
    const s = server({ config: STORED, status: RUNNING });
    setup(s);
    await ready();
    fireEvent.input(screen.getByLabelText('Replace the API key'), { target: { value: 'sk-new' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect((s.writes()[0]?.body as { adapter: Record<string, unknown> }).adapter['apiKey']).toBe('sk-new');
    await waitFor(() => expect(screen.getByTestId('save-bar')).toHaveTextContent('No unsaved changes'));
    fireEvent.click(screen.getByRole('button', { name: 'Remove the stored key' }));
    expect(screen.getByTestId('assist-api-key-stored')).toHaveTextContent('The stored key will be removed when you save.');
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(2));
    expect((s.writes()[1]?.body as { adapter: Record<string, unknown> }).adapter['apiKey']).toBe('');
    await waitFor(() => expect(screen.getByLabelText('API key')).toBeInTheDocument());
  });

  it('does not offer the stored key for a new base URL, and an empty key field there means no key', async () => {
    const s = server({ config: STORED, status: RUNNING });
    setup(s);
    await ready();
    fireEvent.input(screen.getByRole('textbox', { name: 'Base URL' }), { target: { value: 'https://elsewhere.example/v1' } });
    expect(screen.queryByTestId('assist-api-key-stored')).not.toBeInTheDocument();
    expect(screen.getByLabelText('API key')).toHaveValue('');
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect((s.writes()[0]?.body as { adapter: Record<string, unknown> }).adapter['apiKey']).toBe('');
  });
});

describe('admin assist screen: local program', () => {
  const PROGRAMS = ['/opt/llm/run', '/opt/llm/other'];
  const NO_ENDPOINT = { ...RUNNING, running: false, endpointHost: null };

  it('does not offer a local program while the operator lists none, and says how to enable one', async () => {
    setup(server({ config: { ...STORED, adapter: null }, status: NO_ENDPOINT }));
    await ready();
    const kinds = Array.from(screen.getByRole('combobox', { name: 'Endpoint type' }).querySelectorAll('option')).map((o) => o.value);
    expect(kinds).toEqual(['none', 'open-ai-compatible', 'anthropic']);
    expect(screen.getByRole('combobox', { name: 'Endpoint type' })).toHaveAccessibleDescription(/MW_ASSIST_LOCAL_PROGRAMS/);
  });

  it('offers the listed programs only, with arguments as a list, and saves the choice', async () => {
    const s = server({ config: { ...STORED, adapter: null }, status: { ...NO_ENDPOINT, localPrograms: PROGRAMS }, localPrograms: PROGRAMS });
    setup(s);
    await ready();
    fireEvent.change(screen.getByRole('combobox', { name: 'Endpoint type' }), { target: { value: 'local-process' } });
    const program = screen.getByRole('combobox', { name: 'Program' });
    expect(Array.from(program.querySelectorAll('option')).map((o) => o.value)).toEqual(PROGRAMS);
    fireEvent.change(program, { target: { value: '/opt/llm/other' } });
    const args = screen.getByRole('textbox', { name: 'Arguments' });
    for (const a of ['--model', 'small model']) {
      fireEvent.input(args, { target: { value: a } });
      fireEvent.keyDown(args, { key: 'Enter' });
    }
    fireEvent.click(saveButton());
    await waitFor(() => expect(s.writes()).toHaveLength(1));
    expect((s.writes()[0]?.body as { adapter: unknown }).adapter).toEqual({ kind: 'local-process', program: '/opt/llm/other', args: ['--model', 'small model'] });
  });

  it('shows why a stored program that is no longer listed is not running, and refuses to save it before any request', async () => {
    const stored = { ...STORED, adapter: { kind: 'local-process', program: '/usr/bin/old', args: [] } };
    const s = server({ config: stored, status: { ...NO_ENDPOINT, stoppedReason: 'local-program-not-allowed', localPrograms: PROGRAMS }, localPrograms: PROGRAMS });
    setup(s);
    await ready();
    expect(screen.getByTestId('assist-status')).toHaveTextContent(
      'The AI assistant is not running. The configured program /usr/bin/old is not listed in MW_ASSIST_LOCAL_PROGRAMS on the server, so the server did not start it.',
    );
    expect(screen.getByRole('combobox', { name: 'Program' })).toHaveValue('/usr/bin/old');
    fireEvent.click(screen.getByRole('checkbox', { name: 'Recap' }));
    fireEvent.click(saveButton());
    await waitFor(() => expect(screen.getByText(/This program is not listed in MW_ASSIST_LOCAL_PROGRAMS/)).toBeInTheDocument());
    expect(s.writes()).toEqual([]);
  });
});

describe('admin assist screen: rate limit', () => {
  it('is not offered on a server without the settings model, and says where it is set', async () => {
    setup(server({ config: STORED, status: RUNNING }));
    await ready();
    expect(screen.queryByRole('spinbutton', { name: /Requests per minute/ })).toBeNull();
    expect(screen.getByText(/MW_ASSIST_RATE_LIMIT_PER_MIN where the server starts/)).toBeInTheDocument();
  });

  it('writes only the setting, with If-Match, when only the rate limit changed', async () => {
    const s = server({ config: STORED, status: RUNNING });
    const rate: RateLimitApi = {
      load: async () => ({ kind: 'ok', setting: rateSetting(), etag: '"r1"' }),
      save: vi.fn(async (value: number) => ({ setting: rateSetting({ globalValue: value, effective: { value, source: 'global' }, etag: '"r2"' }), etag: '"r2"' })),
    };
    setup(s, rate);
    await ready();
    const field = screen.getByRole('spinbutton', { name: /Requests per minute/ });
    expect(field).toHaveValue(30);
    fireEvent.input(field, { target: { value: '120' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(rate.save).toHaveBeenCalledWith(120, '"r1"'));
    expect(s.writes()).toEqual([]);
  });

  it('a value pinned by MW_ASSIST_RATE_LIMIT_PER_MIN is disabled with its reason', async () => {
    const rate: RateLimitApi = { load: async () => ({ kind: 'ok', setting: rateSetting({ envState: { var: 'MW_ASSIST_RATE_LIMIT_PER_MIN', precedence: 'pin', set: true, value: 10 } }), etag: null }), save: vi.fn() };
    setup(server({ config: STORED, status: RUNNING }), rate);
    await ready();
    expect(screen.getByRole('spinbutton', { name: /Requests per minute/ })).toBeDisabled();
  });

  it('a stale write shows who changed it', async () => {
    const rate: RateLimitApi = {
      load: async () => ({ kind: 'ok', setting: rateSetting(), etag: '"r1"' }),
      save: vi.fn(async () => {
        throw problem(412, { code: 'stale', updatedBy: 'alice', updatedAt: '2026-10-25T10:00:00Z' });
      }),
    };
    setup(server({ config: STORED, status: RUNNING }), rate);
    await ready();
    fireEvent.input(screen.getByRole('spinbutton', { name: /Requests per minute/ }), { target: { value: '5' } });
    fireEvent.click(saveButton());
    await waitFor(() => expect(screen.getByText(/alice/)).toBeInTheDocument());
  });
});

describe('assist values', () => {
  it('round-trips the stored config, leaving the key out', () => {
    const c = parseAdminAssistConfig(STORED) as AdminAssistConfig;
    const out = assistConfig(assistValues(c), true, 'open-ai-compatible https://llm.example.test/v1');
    expect(out).toEqual({ ...STORED, adapter: { kind: 'open-ai-compatible', baseUrl: 'https://llm.example.test/v1', chatModel: 'chat-t28', embedModel: 'embed-t28', sttModel: 'stt-t28' } });
  });

  it('parseAdminAssistConfig accepts the server shape and nothing that only resembles it', () => {
    expect(parseAdminAssistConfig(STORED)).toEqual(STORED);
    expect(parseAdminAssistConfig({ ...STORED, adapter: null }).adapter).toBeNull();
    for (const bad of [null, { enabled: false }, { ...STORED, capabilityGrants: ['send'] }, { ...STORED, adapter: { OpenAiCompatible: { base_url: 'http://mock' } } }, { ...STORED, enabled: 'yes' }]) {
      expect(() => parseAdminAssistConfig(bad)).toThrow();
    }
  });
});
