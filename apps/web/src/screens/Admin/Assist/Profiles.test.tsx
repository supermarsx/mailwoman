import { describe, it, expect, vi, afterEach } from 'vitest';
import { fireEvent, screen, waitFor, cleanup, within } from '@solidjs/testing-library';
import { AssistProfiles, draftOf, type AssignmentApi } from './Profiles.tsx';
import { destinationOf, type Preset, type Profile, type ProfilesApi, type TestOutcome } from './profilesService.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';

afterEach(() => cleanup());

// The shapes mirror `crates/mw-server/src/assist_profiles.rs` (`profile_wire`,
// `PRESETS`, `TestOutcome`); the server side of the same round trip is
// `crates/mw-server/tests/t39_assist_profiles.rs`. This is a unit test of the
// panel against an injected client.

const PRESETS: Preset[] = [
  { id: 'openai-compatible', label: 'OpenAI-compatible (any endpoint)', dialect: 'openai', endpoint: '', models: [], keyRequired: false, local: false, verification: 'mock-wire' },
  { id: 'anthropic', label: 'Anthropic', dialect: 'anthropic', endpoint: 'https://api.anthropic.com', models: ['claude-opus-5-5', 'claude-sonnet-5-5', 'claude-haiku-5-5'], keyRequired: true, local: false, verification: 'mock-wire' },
  { id: 'ollama', label: 'Ollama (local)', dialect: 'openai', endpoint: 'http://127.0.0.1:11434/v1', models: ['llama3.2'], keyRequired: false, local: true, verification: 'unverified' },
];

const STORED: Profile = {
  id: 'p-abc123def456',
  name: 'Claude',
  preset: 'anthropic',
  dialect: 'anthropic',
  endpoint: 'https://api.anthropic.com',
  model: 'claude-sonnet-5-5',
  params: { maxTokens: 2000, anthropicVersion: '2023-06-01' },
  hasApiKey: true,
  destination: 'api.anthropic.com:443',
  egress: 'public',
  updatedBy: 'root',
  updatedAt: 1,
};

function api(over: Partial<ProfilesApi> = {}): ProfilesApi {
  return {
    presets: vi.fn(async () => PRESETS),
    list: vi.fn(async () => [STORED]),
    create: vi.fn(async () => STORED),
    update: vi.fn(async () => STORED),
    remove: vi.fn(async () => {}),
    test: vi.fn(async (): Promise<TestOutcome> => ({ ok: true, latencyMs: 412, reply: 'OK', error: null })),
    grants: vi.fn(async () => []),
    addGrant: vi.fn(async () => ({ id: 'g-1', destination: '127.0.0.1:11434', note: '', createdBy: 'root', createdAt: 1 })),
    removeGrant: vi.fn(async () => {}),
    ...over,
  };
}

const SETTING = {
  key: 'assist.profile', family: false, kind: { type: 'text', format: 'slug', maxLength: 64 }, default: '', nullable: false,
  scopes: 'global+domain+admin_user', userOverride: false, category: 'assistant', mapsTo: '', env: null, dedicatedRoute: null,
  scopeId: 'account', hot: true, ceiling: false, userVisible: false, globalValue: '', allowUserOverride: false, globalLocked: false,
  updatedBy: null, updatedAt: null, effective: { value: '', source: 'default' }, envState: null, overrides: {}, etag: 'e1',
} as unknown as AdminSetting;

function noAssignments(): AssignmentApi {
  return { load: vi.fn(async () => null), saveDefault: vi.fn() };
}

describe('AssistProfiles', () => {
  it('lists profiles without any key and runs Test profile with the latency', async () => {
    const a = api();
    renderInShell(() => <AssistProfiles api={a} assignments={noAssignments()} />);
    const row = await screen.findByTestId(`assist-profile-${STORED.id}`);
    expect(within(row).getByText('Stored')).toBeTruthy();
    expect(row.textContent).not.toContain('sk-');
    fireEvent.click(within(row).getByRole('button', { name: 'Test profile' }));
    await waitFor(() => expect(screen.getByTestId(`assist-profile-test-${STORED.id}`).textContent).toContain('412'));
    expect(a.test).toHaveBeenCalledWith(STORED.id);
  });

  it('shows a classified failure with the status and provider code, never a message', async () => {
    const a = api({ test: vi.fn(async () => ({ ok: false, latencyMs: 30, reply: null, error: { kind: 'unauthorized' as const, status: 401, upstreamCode: 'invalid_api_key' } })) });
    renderInShell(() => <AssistProfiles api={a} assignments={noAssignments()} />);
    const row = await screen.findByTestId(`assist-profile-${STORED.id}`);
    fireEvent.click(within(row).getByRole('button', { name: 'Test profile' }));
    await waitFor(() => {
      const text = screen.getByTestId(`assist-profile-test-${STORED.id}`).textContent ?? '';
      expect(text).toContain('the provider refused the API key');
      expect(text).toContain('401 invalid_api_key');
    });
  });

  it('starts a new profile from a preset and says how far the preset was checked', async () => {
    const a = api();
    renderInShell(() => <AssistProfiles api={a} assignments={noAssignments()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add profile' }));
    const editor = await screen.findByTestId('assist-profile-editor');
    fireEvent.change(within(editor).getByLabelText('Preset'), { target: { value: 'ollama' } });
    expect((within(editor).getByLabelText('Endpoint URL') as HTMLInputElement).value).toBe('http://127.0.0.1:11434/v1');
    expect((within(editor).getByLabelText('Model') as HTMLInputElement).value).toBe('llama3.2');
    expect(screen.getByTestId('assist-preset-verification').textContent).toContain('Not tested');
    expect(screen.getByTestId('assist-profile-needs-grant').textContent).toContain('127.0.0.1:11434');
    fireEvent.change(within(editor).getByLabelText('Preset'), { target: { value: 'anthropic' } });
    expect(screen.getByTestId('assist-preset-verification').textContent).toContain('local mock');
    fireEvent.input(within(editor).getByLabelText('Name'), { target: { value: 'Claude Haiku' } });
    fireEvent.input(within(editor).getByLabelText('API key'), { target: { value: 'sk-new' } });
    fireEvent.input(within(editor).getByLabelText('Time limit (ms)'), { target: { value: '30000' } });
    fireEvent.click(within(editor).getByRole('button', { name: 'Save profile' }));
    await waitFor(() => expect(a.create).toHaveBeenCalled());
    expect(a.create).toHaveBeenCalledWith({ name: 'Claude Haiku', preset: 'anthropic', endpoint: 'https://api.anthropic.com', model: 'claude-opus-5-5', params: { timeoutMs: 30000 }, apiKey: 'sk-new' });
  });

  it('editing keeps the stored key unless one is typed or removal is asked', () => {
    const form = { name: 'Claude', preset: 'anthropic', endpoint: 'https://api.anthropic.com', model: 'm', apiKey: '', removeKey: false, maxTokens: '', temperature: '0.5', timeoutMs: '' };
    const keep = draftOf(form, STORED);
    expect('apiKey' in keep).toBe(false);
    // Fields this form does not edit are kept; cleared ones are dropped.
    expect(keep.params).toEqual({ anthropicVersion: '2023-06-01', temperature: 0.5 });
    expect(draftOf({ ...form, removeKey: true }, STORED).apiKey).toBe('');
    expect(draftOf({ ...form, apiKey: 'sk-x' }, STORED).apiKey).toBe('sk-x');
  });

  it('saves the deployment default with its lock', async () => {
    const assignments: AssignmentApi = { load: vi.fn(async () => ({ setting: SETTING, etag: 'e1' })), saveDefault: vi.fn(async () => ({ setting: SETTING, etag: 'e2' })) };
    renderInShell(() => <AssistProfiles api={api()} assignments={assignments} />);
    await screen.findByTestId(`assist-profile-${STORED.id}`);
    const select = await screen.findByLabelText('Deployment default');
    fireEvent.change(select, { target: { value: STORED.id } });
    fireEvent.click(screen.getByLabelText('Locked: domains and users cannot use another profile'));
    fireEvent.click(screen.getByRole('button', { name: 'Save default' }));
    await waitFor(() => expect(assignments.saveDefault).toHaveBeenCalledWith(STORED.id, true, 'e1'));
  });

  it('adds a local endpoint grant', async () => {
    const a = api();
    renderInShell(() => <AssistProfiles api={a} assignments={noAssignments()} />);
    await screen.findByTestId(`assist-profile-${STORED.id}`);
    fireEvent.input(screen.getByLabelText('Host and port'), { target: { value: ' 127.0.0.1:11434 ' } });
    fireEvent.input(screen.getByLabelText('Note'), { target: { value: 'ollama box' } });
    fireEvent.click(screen.getByRole('button', { name: 'Grant' }));
    await waitFor(() => expect(a.addGrant).toHaveBeenCalledWith('127.0.0.1:11434', 'ollama box'));
  });

  it('has no axe violations and no inline styles', async () => {
    const { container } = renderInShell(() => <AssistProfiles api={api()} assignments={noAssignments()} />);
    await screen.findByTestId(`assist-profile-${STORED.id}`);
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });

  it('spells destinations as the server does', () => {
    expect(destinationOf('http://127.0.0.1:11434/v1')).toBe('127.0.0.1:11434');
    expect(destinationOf('https://API.example.com/v1')).toBe('api.example.com:443');
    expect(destinationOf('http://[::1]:1234/v1')).toBe('[::1]:1234');
    expect(destinationOf('nope')).toBeNull();
  });
});
