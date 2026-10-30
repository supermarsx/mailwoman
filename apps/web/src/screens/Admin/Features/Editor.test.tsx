// t98: the grouped feature profile editor — search, per-group allow/disallow,
// presets, the diff against the server default, JSON import/export, clone and
// the effective-value preview.
import { describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminFeatures, NEW } from './index.tsx';
import { PROFILE_FORMAT, type EffectivePreview, type FeatureCatalogue, type FeaturesApi, type ProfileAssignment, type ProfileValue } from './api.ts';
import { applyToGroup, diffAgainstDefault, groupFeatures, readDocument, readInput } from './model.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

const REV = (n: number): string => `"fg1_${String(n).repeat(64).slice(0, 64)}"`;
const v = (value: unknown, locked = false): ProfileValue => ({ value, locked });

function catalogue(): FeatureCatalogue {
  return {
    revision: 'fg1_1',
    profiles: [{ id: 'work', name: 'Work', values: { 'cap.pim.calendar': v(false), 'cap.compose.rich_text': v(true) }, builtin: false, updatedAt: 1 }],
    groups: [],
    assignments: [],
    memberships: {},
    nativeTargets: [{ accountId: 'acct-1', address: 'ana@example.test', domain: 'example.test' }],
    domains: ['example.test'],
    categories: ['compose', 'pim'],
    catalogue: [
      { id: 'compose.rich_text', category: 'compose', control: 'toggle', capabilityKeys: ['cap.compose.rich_text'], limitKeys: [], settingKeys: [], enforcement: 'enforced' },
      { id: 'compose.attachments', category: 'compose', control: 'toggle', capabilityKeys: ['cap.compose.attachments'], limitKeys: ['limit.attachment_bytes'], settingKeys: [], enforcement: 'enforced' },
      { id: 'calendar', category: 'pim', control: 'toggle', capabilityKeys: ['cap.pim.calendar'], limitKeys: [], settingKeys: [], enforcement: 'enforced' },
      { id: 'tasks', category: 'pim', control: 'toggle', capabilityKeys: ['cap.pim.tasks'], limitKeys: [], settingKeys: [], enforcement: 'enforced' },
      { id: 'views.attachments', category: 'pim', control: 'toggle', capabilityKeys: ['cap.views.attachments'], limitKeys: [], settingKeys: [], enforcement: 'ui' },
    ],
    profileKeys: [
      { key: 'limit.attachment_bytes', kind: { type: 'int', min: 0, max: 1000000000 }, default: 50000000, category: 'limits' },
      { key: 'sieve.maxScripts', kind: { type: 'int', min: 1, max: 100 }, default: 5, category: 'sieve' },
    ],
    defaults: { 'cap.compose.rich_text': true, 'cap.compose.attachments': true, 'cap.pim.calendar': true, 'cap.pim.tasks': true, 'limit.attachment_bytes': 50000000 },
    presets: [
      { id: 'mail-only', values: { 'cap.pim.calendar': v(false), 'cap.pim.tasks': v(false), 'cap.compose.rich_text': v(true) } },
      { id: 'full', values: { 'cap.pim.calendar': v(true), 'cap.pim.tasks': v(true) } },
    ],
  };
}

interface MockApi extends FeaturesApi {
  calls: string[];
  state: FeatureCatalogue;
  imported: unknown;
}

function mockApi(): MockApi {
  let rev = 1;
  const api: MockApi = {
    calls: [],
    state: catalogue(),
    imported: null,
    async load() {
      return { data: structuredClone(api.state), etag: REV(rev) };
    },
    async putProfile(id: string, body: { name: string; values: Record<string, ProfileValue> }) {
      api.calls.push(`put:${id}:${JSON.stringify(body)}`);
      api.state.profiles = [...api.state.profiles.filter((p) => p.id !== id), { id, name: body.name, values: body.values, builtin: false, updatedAt: 2 }];
      return REV(++rev);
    },
    async deleteProfile() {
      return REV(++rev);
    },
    async assign(_a: ProfileAssignment) {
      return REV(++rev);
    },
    async unassign(_a: ProfileAssignment) {
      return REV(++rev);
    },
    async importDocument(doc: unknown) {
      api.imported = doc;
      const d = doc as { name: string; values: Record<string, ProfileValue> };
      return { name: d.name, values: d.values };
    },
    async effective(subject: { account: string } | { domain: string }): Promise<EffectivePreview> {
      api.calls.push(`effective:${JSON.stringify(subject)}`);
      return {
        subject: { account: 'acct-1', address: 'ana@example.test', domain: 'example.test' },
        profiles: ['work'],
        groups: [],
        features: [
          { id: 'calendar', category: 'pim', enabled: false, enforcement: 'enforced', keys: { 'cap.pim.calendar': { value: false, source: 'profile', locked: false, profile: 'work' } } },
          { id: 'tasks', category: 'pim', enabled: true, enforcement: 'enforced', keys: { 'cap.pim.tasks': { value: true, source: 'default', locked: false } } },
        ],
      };
    },
  };
  return api;
}

function setup(rest: string[]) {
  const api = mockApi();
  const [route, setRoute] = createSignal<AdminRoute>({ section: 'features', rest });
  const navigate = vi.fn((path: string) => {
    const [, ...parts] = (path.split('#/')[1] ?? path).split('/');
    setRoute({ section: 'features', rest: parts.map(decodeURIComponent) });
  });
  const utils = renderInShell(() => <AdminFeatures api={api} route={route()} navigate={navigate} />);
  return { ...utils, api, navigate };
}

const readBlob = (b: Blob): Promise<string> => new Promise((resolve) => { const r = new FileReader(); r.onload = () => resolve(String(r.result)); r.readAsText(b); });

const lastPut = (api: MockApi): Record<string, ProfileValue> => {
  const call = [...api.calls].reverse().find((c) => c.startsWith('put:'))!;
  return (JSON.parse(call.slice(call.indexOf('{'))) as { values: Record<string, ProfileValue> }).values;
};

describe('Admin → Feature profiles editor (t98)', () => {
  it('groups features by category, filters by search and marks what differs from the default', async () => {
    const { container } = setup(['work']);
    const pim = await waitFor(() => {
      const el = container.querySelector('[data-category="pim"]');
      expect(el).not.toBeNull();
      return el as HTMLElement;
    });
    expect(container.querySelector('[data-category="compose"]')).not.toBeNull();
    expect(pim.querySelector('[data-feature="calendar"]')!.getAttribute('data-differs')).toBe('true');
    expect(container.querySelector('[data-feature="compose.rich_text"]')!.getAttribute('data-differs')).toBe('false');
    expect(within(pim).getByText('Hidden in the app only')).toBeInTheDocument();
    expect(stripIsolates(screen.getByTestId('feature-diff-summary').textContent ?? '')).toBe('1 value differs from the server default · 1 repeat it');
    fireEvent.input(screen.getByLabelText('Find a feature'), { target: { value: 'tasks' } });
    await waitFor(() => expect(container.querySelector('[data-feature="compose.rich_text"]')).toBeNull());
    expect(container.querySelector('[data-feature="tasks"]')).not.toBeNull();
    // The category name matches too, so the whole group stays.
    expect(container.querySelector('[data-feature="compose.attachments"]')).toBeNull();
    await expectNoAxeViolations(container);
  });

  it('allows or disallows a whole group at once and saves typed limits', async () => {
    const { api } = setup(['work']);
    fireEvent.click(await screen.findByRole('button', { name: 'Disallow every feature in Mail and compose' }));
    fireEvent.input(screen.getByLabelText('Find a feature'), { target: { value: '' } });
    const limit = screen.getByLabelText('Attachment size limit');
    fireEvent.change(limit, { target: { value: '1000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('put:work:'))).toBe(true));
    const values = lastPut(api);
    expect(values['cap.compose.rich_text']).toEqual(v(false));
    expect(values['cap.compose.attachments']).toEqual(v(false));
    expect(values['limit.attachment_bytes']).toEqual(v(1000));
    // The other group is untouched.
    expect(values['cap.pim.calendar']).toEqual(v(false));
    expect(values['cap.pim.tasks']).toBeUndefined();
  });

  it('applies a preset to a new profile and names it', async () => {
    const { api, navigate } = setup([NEW]);
    fireEvent.change(await screen.findByTestId('feature-preset'), { target: { value: 'mail-only' } });
    fireEvent.click(screen.getByRole('button', { name: 'Apply preset' }));
    expect((screen.getByLabelText('Profile name') as HTMLInputElement).value).toBe('Mail only');
    expect((screen.getByLabelText('Value for Calendar') as HTMLSelectElement).value).toBe('off');
    fireEvent.click(screen.getByRole('button', { name: 'Create profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('put:mail-only:'))).toBe(true));
    expect(lastPut(api)['cap.pim.tasks']).toEqual(v(false));
    await waitFor(() => expect(navigate).toHaveBeenCalled());
  });

  it('imports a profile document through the server check and refuses other files', async () => {
    const { api } = setup([NEW]);
    const input = (await screen.findByTestId('feature-import-file')) as HTMLInputElement;
    const doc = { format: PROFILE_FORMAT, version: 1, name: 'Imported', values: { 'cap.pim.tasks': v(false, true) } };
    fireEvent.change(input, { target: { files: [new File([JSON.stringify(doc)], 'p.json', { type: 'application/json' })] } });
    await waitFor(() => expect((screen.getByLabelText('Profile name') as HTMLInputElement).value).toBe('Imported'));
    expect(api.imported).toEqual(doc);
    expect((screen.getByLabelText('Value for Tasks') as HTMLSelectElement).value).toBe('off');
    expect((screen.getByRole('checkbox', { name: /Locked\s+Tasks/ }) as HTMLInputElement).checked).toBe(true);
    fireEvent.change(input, { target: { files: [new File(['{"format":"other"}'], 'x.json')] } });
    expect(await screen.findByText('The file is not a Mailwoman feature profile.')).toBeInTheDocument();
  });

  it('exports the current values as a profile document', async () => {
    const created: Blob[] = [];
    const urlApi = URL as unknown as { createObjectURL?: ((b: Blob) => string) | undefined; revokeObjectURL?: ((u: string) => void) | undefined };
    const before = { create: urlApi.createObjectURL, revoke: urlApi.revokeObjectURL };
    urlApi.createObjectURL = (b: Blob) => { created.push(b); return 'blob:x'; };
    urlApi.revokeObjectURL = () => {};
    try {
      setup(['work']);
      fireEvent.click(await screen.findByRole('button', { name: 'Export JSON' }));
      expect(created).toHaveLength(1);
      const doc = JSON.parse(await readBlob(created[0]!)) as { format: string; version: number; name: string; values: Record<string, ProfileValue> };
      expect(doc).toEqual({ format: PROFILE_FORMAT, version: 1, name: 'Work', values: { 'cap.pim.calendar': v(false), 'cap.compose.rich_text': v(true) } });
    } finally {
      urlApi.createObjectURL = before.create;
      urlApi.revokeObjectURL = before.revoke;
    }
  });

  it('clones a profile into the new-profile form in place', async () => {
    const { api, navigate } = setup(['work']);
    fireEvent.click(await screen.findByRole('button', { name: 'Clone' }));
    await waitFor(() => expect(navigate).toHaveBeenCalledWith(sectionPath('features', [NEW])));
    await waitFor(() => expect((screen.getByLabelText('Profile name') as HTMLInputElement).value).toContain('Work'));
    expect((screen.getByLabelText('Value for Calendar') as HTMLSelectElement).value).toBe('off');
    fireEvent.click(screen.getByRole('button', { name: 'Create profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('put:copy-of-work:'))).toBe(true));
  });

  it('previews the effective values for a user with where each comes from', async () => {
    const { api, container } = setup(['work']);
    const preview = await screen.findByTestId('feature-preview');
    fireEvent.click(within(preview).getByRole('button', { name: 'Show' }));
    await waitFor(() => expect(container.querySelector('[data-preview-feature="calendar"]')).not.toBeNull());
    expect(api.calls).toContain('effective:{"account":"acct-1"}');
    const row = container.querySelector('[data-preview-feature="calendar"]') as HTMLElement;
    expect(row.getAttribute('data-enabled')).toBe('false');
    expect(stripIsolates(row.textContent ?? '')).toContain('Profile Work');
    expect(stripIsolates(screen.getByTestId('feature-preview-profiles').textContent ?? '')).toContain('Work');
    await expectNoAxeViolations(container);
  });
});

describe('feature profile model', () => {
  it('groups in the server order and applies a group action', () => {
    const c = catalogue();
    expect(groupFeatures(c, () => true).map((g) => g.category)).toEqual(['compose', 'pim']);
    const pim = groupFeatures(c, () => true)[1]!.features;
    const on = applyToGroup({ 'cap.pim.tasks': v(false, true) }, pim, 'on');
    expect(on['cap.pim.tasks']).toEqual(v(true, true));
    expect(on['cap.pim.calendar']).toEqual(v(true));
    expect(applyToGroup(on, pim, 'clear')).toEqual({});
  });
  it('diffs against the default; a lock always counts as a difference', () => {
    expect(diffAgainstDefault({ a: v(true), b: v(false), c: v(true, true) }, { a: true, b: true, c: true })).toEqual({ differs: ['b', 'c'], same: ['a'] });
  });
  it('reads typed inputs and refuses bad ones', () => {
    expect(readInput({ type: 'int', min: 0, max: 10 }, ' 7 ')).toBe(7);
    expect(readInput({ type: 'int', min: 0, max: 10 }, '11')).toBeInstanceOf(Error);
    expect(readInput({ type: 'int' }, '')).toBeUndefined();
    expect(readInput({ type: 'stringList' }, 'imap, ews')).toEqual(['imap', 'ews']);
    expect(readDocument('nope')).toEqual({ ok: false, reason: 'json' });
    expect(readDocument(JSON.stringify({ format: PROFILE_FORMAT, version: 2 }))).toEqual({ ok: false, reason: 'version' });
  });
});
