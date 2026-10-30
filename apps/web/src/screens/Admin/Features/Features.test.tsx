import { describe, expect, it, vi } from 'vitest';
import { createSignal } from 'solid-js';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminFeatures, NEW } from './index.tsx';
import { profileIdFor, type FeatureCatalogue, type FeaturesApi, type ProfileAssignment } from './api.ts';
import { AdminProblemError } from '../shell/adminApi.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

const REV = (n: number): string => `"fg1_${String(n).repeat(64).slice(0, 64)}"`;

function catalogue(): FeatureCatalogue {
  return {
    revision: 'fg1_1',
    profiles: [{ id: 'work', name: 'Work', values: { 'cap.export': { value: false, locked: true }, 'limit.other': { value: 5, locked: false } }, builtin: false, updatedAt: 1 }],
    groups: [{ id: 'g1', name: 'Sales', domain: 'example.test', source: 'manual' }],
    assignments: [{ scopeKind: 'domain', scopeId: 'example.test', profileId: 'work', priority: 1 }],
    memberships: { g1: [] },
    nativeTargets: [{ accountId: 'acct-1', address: 'ana@example.test', domain: 'example.test' }],
    domains: ['example.test', 'other.test'],
    catalogue: [
      { id: 'export', capabilityKeys: ['cap.export'], limitKeys: [], enforcement: 'enforced' },
      { id: 'contacts', capabilityKeys: ['cap.pim.contacts'], limitKeys: [], enforcement: 'enforced' },
    ],
  };
}

function mockApi(): FeaturesApi & { calls: string[]; state: FeatureCatalogue; stale: boolean } {
  let rev = 1;
  const api = {
    calls: [] as string[],
    state: catalogue(),
    stale: false,
    async load() {
      return { data: structuredClone(api.state), etag: REV(rev) };
    },
    async putProfile(id: string, body: { name: string; values: Record<string, { value: unknown; locked: boolean }> }, etag: string) {
      if (api.stale) { api.stale = false; rev++; throw new AdminProblemError(412, { code: 'preconditionFailed' }, null); }
      api.calls.push(`put:${id}:${etag}:${JSON.stringify(body)}`);
      const rest = api.state.profiles.filter((p) => p.id !== id);
      api.state.profiles = [...rest, { id, name: body.name, values: body.values, builtin: false, updatedAt: 2 }];
      return REV(++rev);
    },
    async deleteProfile(id: string, etag: string) {
      api.calls.push(`delete:${id}:${etag}`);
      api.state.profiles = api.state.profiles.filter((p) => p.id !== id);
      api.state.assignments = api.state.assignments.filter((a) => a.profileId !== id);
      return REV(++rev);
    },
    async assign(a: ProfileAssignment, etag: string) {
      api.calls.push(`assign:${JSON.stringify(a)}:${etag}`);
      api.state.assignments = [...api.state.assignments, a];
      return REV(++rev);
    },
    async unassign(a: ProfileAssignment, etag: string) {
      api.calls.push(`unassign:${a.scopeKind}:${a.scopeId}:${etag}`);
      api.state.assignments = api.state.assignments.filter((x) => !(x.scopeKind === a.scopeKind && x.scopeId === a.scopeId && x.profileId === a.profileId));
      return REV(++rev);
    },
  };
  return api;
}

function setup(rest: string[] = []) {
  const api = mockApi();
  const [route, setRoute] = createSignal<AdminRoute>({ section: 'features', rest });
  const navigate = vi.fn((path: string) => {
    const [, ...parts] = path.split('/');
    setRoute({ section: 'features', rest: parts.map(decodeURIComponent) });
  });
  const utils = renderInShell(() => <AdminFeatures api={api} route={route()} navigate={navigate} />);
  return { ...utils, api, navigate, setRoute };
}

describe('Admin → Feature profiles', () => {
  it('lists profiles with their switch and assignment counts', async () => {
    const { container } = setup();
    const row = await waitFor(() => {
      const el = container.querySelector('[data-row-key="work"]');
      expect(el).not.toBeNull();
      return el as HTMLElement;
    });
    expect(within(row).getByText('Work')).toBeInTheDocument();
    expect(within(row).getByText('2 switches set')).toBeInTheDocument();
    expect(within(row).getByText('1 place')).toBeInTheDocument();
    await expectNoAxeViolations(container);
  });

  it('edits a profile switch and keeps values outside the catalogue, sending the read revision', async () => {
    const { api, container } = setup(['work']);
    const select = await screen.findByLabelText('Value for Export');
    expect((select as HTMLSelectElement).value).toBe('off');
    fireEvent.change(screen.getByLabelText('Value for Contacts'), { target: { value: 'on' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('put:work:'))).toBe(true));
    const call = api.calls.find((c) => c.startsWith('put:work:'))!;
    expect(call).toContain(REV(1));
    const body = JSON.parse(call.slice(call.indexOf('{'))) as { values: Record<string, unknown> };
    expect(body.values).toEqual({
      'cap.export': { value: false, locked: true },
      'cap.pim.contacts': { value: true, locked: false },
      'limit.other': { value: 5, locked: false },
    });
    expect(stripIsolates((await screen.findByTestId('features-notice')).textContent ?? '')).toContain('Profile Work saved.');
    await expectNoAxeViolations(container);
  });

  it('creates a profile from the new form and opens it', async () => {
    const { api, navigate } = setup([NEW]);
    fireEvent.input(await screen.findByLabelText('Profile name'), { target: { value: 'Restricted staff' } });
    fireEvent.change(screen.getByLabelText('Value for Export'), { target: { value: 'off' } });
    fireEvent.click(screen.getByRole('checkbox', { name: /Locked\s+Export/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Create profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('put:restricted-staff:'))).toBe(true));
    expect(api.calls[0]).toContain('"cap.export":{"value":false,"locked":true}');
    await waitFor(() => expect(navigate).toHaveBeenCalledWith(sectionPath('features', ['restricted-staff'])));
  });

  it('refuses an empty name without a request', async () => {
    const { api } = setup([NEW]);
    fireEvent.click(await screen.findByRole('button', { name: 'Create profile' }));
    expect(await screen.findByText('Enter a name of 1 to 128 characters.')).toBeInTheDocument();
    expect(api.calls).toEqual([]);
  });

  it('assigns to a user and a group, and removes a domain assignment', async () => {
    const { api } = setup(['work']);
    const form = await screen.findByRole('form', { name: 'Assign this profile' });
    fireEvent.change(within(form).getByRole('combobox', { name: 'Applies to' }), { target: { value: 'user' } });
    expect((within(form).getByRole('combobox', { name: 'Name' }) as HTMLSelectElement).value).toBe('acct-1');
    fireEvent.input(within(form).getByRole('spinbutton', { name: 'Priority' }), { target: { value: '5' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Assign' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('assign:'))).toBe(true));
    expect(api.calls[0]).toContain('{"scopeKind":"user","scopeId":"acct-1","profileId":"work","priority":5}');
    expect(await screen.findByText('ana@example.test', { selector: 'th' })).toBeInTheDocument();
    // The detail re-renders from the reloaded catalogue.
    const again = screen.getByRole('form', { name: 'Assign this profile' });
    fireEvent.change(within(again).getByRole('combobox', { name: 'Applies to' }), { target: { value: 'group' } });
    expect(within(again).getByRole('option', { name: 'Sales (example.test)' })).toBeInTheDocument();
    // The detail stays mounted across the reload (t98), so wait for the assign to settle.
    const remove = screen.getByRole('button', { name: 'Remove example.test' });
    await waitFor(() => expect(remove).not.toBeDisabled());
    fireEvent.click(remove);
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('unassign:domain:example.test:'))).toBe(true));
  });

  it('reloads and explains a stale catalogue instead of overwriting', async () => {
    const { api } = setup(['work']);
    api.stale = true;
    fireEvent.click(await screen.findByRole('button', { name: 'Save profile' }));
    expect(await screen.findByText(/Another administrator changed the feature catalogue/)).toBeInTheDocument();
    expect(api.calls).toEqual([]);
  });

  it('deletes a profile after confirmation', async () => {
    const { api, navigate } = setup(['work']);
    fireEvent.click(await screen.findByRole('button', { name: 'Delete profile' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('assigned to');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Delete profile' }));
    await waitFor(() => expect(api.calls.some((c) => c.startsWith('delete:work:'))).toBe(true));
    await waitFor(() => expect(navigate).toHaveBeenCalledWith(sectionPath('features')));
  });
});

describe('profileIdFor', () => {
  it('derives a stable id and avoids collisions', () => {
    expect(profileIdFor('Restricted Staff!', [])).toBe('restricted-staff');
    expect(profileIdFor('***', [])).toBe('profile');
    expect(profileIdFor('Work', ['work'])).toMatch(/^work-[a-z0-9]{1,4}$/);
  });
});
