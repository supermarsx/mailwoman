import { describe, it, expect } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { AdminUiPlugins, REGISTER } from './index.tsx';
import { readManifest, sameOrigin } from './ManifestInput.tsx';
import { fileToBase64, UiPluginsApiError, type UiPluginInfo, type UiPluginsApi } from './service.ts';
import { renderInShell } from '../shell/testkit.tsx';
import type { AdminRoute } from '../shell/router.ts';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

// The mock follows crates/mw-server/src/ui_plugins.rs:
//   `list_admin` :346  the row shape (no grant state)
//   `register`   :375  201 `{ id, signed, bannerSignal }`; 403 for an unsigned
//                      manifest without `allowUnsigned` (`verify_bundle` :181)
//   `approve`    :456  approves AND enables (`do_approve` :297)
//   `enable`     :473  403 before approval
//   `do_grant`   :316  400 for a capability the manifest does not declare
//   `remove`     :546

function row(over: Partial<UiPluginInfo> = {}): UiPluginInfo {
  return {
    id: 'acme-toolbar',
    name: 'Acme toolbar',
    version: '1.2.0',
    enabled: false,
    approved: false,
    signed: false,
    capabilities: ['ui:message-toolbar', 'net:host-allowlist'],
    extensionPoints: ['message-toolbar'],
    ...over,
  };
}

function mockApi(initial: UiPluginInfo[] = []): UiPluginsApi & { calls: string[] } {
  const calls: string[] = [];
  let rows = initial;
  const patch = (id: string, over: Partial<UiPluginInfo>): void => {
    rows = rows.map((r) => (r.id === id ? { ...r, ...over } : r));
  };
  return {
    calls,
    async list() {
      return rows;
    },
    async register(manifest, bundle, allowUnsigned) {
      calls.push(`register:${JSON.stringify(manifest)}:${bundle}:${allowUnsigned}`);
      const m = manifest as { id: string; name: string; version: string; signature?: string; capabilities?: string[] };
      if (m.signature === undefined && !allowUnsigned) {
        throw new UiPluginsApiError(403, 'plugin is unsigned and allowUnsigned is not set');
      }
      rows = [...rows, row({ id: m.id, name: m.name, version: m.version, capabilities: m.capabilities ?? [] })];
      return { id: m.id, signed: m.signature !== undefined, bannerSignal: m.signature === undefined };
    },
    async approve(id) {
      calls.push(`approve:${id}`);
      patch(id, { approved: true, enabled: true });
    },
    async enable(id) {
      calls.push(`enable:${id}`);
      if (!rows.find((r) => r.id === id)?.approved) {
        throw new UiPluginsApiError(403, 'plugin must be approved before it can be enabled');
      }
      patch(id, { enabled: true });
    },
    async disable(id) {
      calls.push(`disable:${id}`);
      patch(id, { enabled: false });
    },
    async grant(id, capability, params) {
      calls.push(`grant:${id}:${capability}:${JSON.stringify(params)}`);
      if (!rows.find((r) => r.id === id)?.capabilities.includes(capability)) {
        throw new UiPluginsApiError(400, 'capability not declared by the manifest (deny-by-default)');
      }
    },
    async remove(id) {
      calls.push(`remove:${id}`);
      rows = rows.filter((r) => r.id !== id);
    },
  };
}

const MANIFEST = { id: 'acme-toolbar', name: 'Acme toolbar', version: '1.2.0', capabilities: ['ui:message-toolbar'], extensionPoints: ['message-toolbar'] };

function setup(initial: UiPluginInfo[] = [], rest: string[] = []) {
  const api = mockApi(initial);
  const [route, setRoute] = createSignal<AdminRoute>({ section: 'ui-plugins', rest });
  const navigate = (path: string): void => {
    setRoute({ section: 'ui-plugins', rest: (path.split('#/ui-plugins')[1] ?? '').split('/').filter(Boolean).map(decodeURIComponent) });
  };
  const utils = renderInShell(() => <AdminUiPlugins api={api} route={route()} navigate={navigate} />);
  return { ...utils, api };
}
const listRow = (id: string): HTMLElement => document.querySelector(`[data-row-key="${id}"]`) as HTMLElement;

describe('Admin → UI plugins: registration', () => {
  it('reads a manifest file, previews it, and registers an unsigned one only when allowed', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('ui-plugin-register');
    const file = new File([JSON.stringify(MANIFEST)], 'ui-plugin.json', { type: 'application/json' });
    fireEvent.change(within(form).getByLabelText('Manifest file'), { target: { files: [file] } });
    const preview = await within(form).findByTestId('ui-manifest-preview');
    expect(preview).toHaveTextContent('acme-toolbar');
    expect(preview).toHaveTextContent('ui:message-toolbar');
    expect(preview).toHaveTextContent('Unsigned');
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(within(form).getByText(/plugin is unsigned and allowUnsigned is not set/)).toBeInTheDocument());
    fireEvent.click(within(form).getByRole('checkbox', { name: 'Allow this plugin to register without a signature' }));
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    const card = await waitFor(() => screen.getByTestId('ui-plugin-card'));
    expect(api.calls.at(-1)).toBe(`register:${JSON.stringify(MANIFEST)}:null:true`);
    expect(card).toHaveAttribute('data-plugin-id', 'acme-toolbar');
    expect(stripIsolates(screen.getByTestId('ui-plugins-notice').textContent ?? '')).toMatch(/registered without a signature/);
  });

  it('a manifest that is not JSON, or declares an unknown capability, never reaches the server', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('ui-plugin-register');
    const paste = within(form).getByRole('textbox', { name: 'Or paste the manifest' });
    fireEvent.input(paste, { target: { value: '{ not json' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(within(form).getByText('The manifest is not valid JSON.')).toBeInTheDocument());
    fireEvent.input(paste, { target: { value: JSON.stringify({ ...MANIFEST, capabilities: ['ui:everything'] }) } });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(stripIsolates(within(form).getByText(/which this server does not know/).textContent ?? '')).toMatch(/ui:everything/));
    expect(api.calls).toEqual([]);
  });

  it('a manifest address on another server is refused with the reason, before any fetch', async () => {
    setup([], [REGISTER]);
    const form = await screen.findByTestId('ui-plugin-register');
    fireEvent.input(within(form).getByRole('textbox', { name: /address on this server/ }), { target: { value: 'https://elsewhere.example/ui-plugin.json' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Read' }));
    expect(await within(form).findByRole('alert')).toHaveTextContent('That address is on another server');
  });

  it('a signed manifest needs its bundle', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('ui-plugin-register');
    fireEvent.input(within(form).getByRole('textbox', { name: 'Or paste the manifest' }), { target: { value: JSON.stringify({ ...MANIFEST, signature: 'c2ln' }) } });
    expect(within(form).queryByRole('checkbox', { name: /without a signature/ })).toBeNull();
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(within(form).getByText(/A signed manifest needs its bundle file/)).toBeInTheDocument());
    expect(api.calls).toEqual([]);
  });
});

describe('Admin → UI plugins: list and detail', () => {
  it('a failed read shows the error with Try again', async () => {
    const api = mockApi([row()]);
    let fail = true;
    const list = api.list.bind(api);
    api.list = async () => {
      if (fail) throw new UiPluginsApiError(500, 'boom');
      return list();
    };
    renderInShell(() => <AdminUiPlugins api={api} route={{ section: 'ui-plugins', rest: [] }} navigate={() => undefined} />);
    expect(await screen.findByText('Could not load the UI plugins.')).toBeInTheDocument();
    fail = false;
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    await waitFor(() => expect(listRow('acme-toolbar')).not.toBeNull());
  });

  it('approve enables; disable asks first; enable follows the server', async () => {
    const { api } = setup([row()]);
    await waitFor(() => expect(listRow('acme-toolbar')).not.toBeNull());
    fireEvent.click(within(listRow('acme-toolbar')).getByRole('button', { name: /^Approve and enable/ }));
    await waitFor(() => expect(within(listRow('acme-toolbar')).getByTestId('ui-enabled-chip')).toHaveTextContent('Enabled'));
    fireEvent.click(within(listRow('acme-toolbar')).getByRole('button', { name: /^Disable/ }));
    fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Disable' }));
    await waitFor(() => expect(within(listRow('acme-toolbar')).getByTestId('ui-enabled-chip')).toHaveTextContent('Disabled'));
    fireEvent.click(within(listRow('acme-toolbar')).getByRole('button', { name: /^Enable/ }));
    await waitFor(() => expect(within(listRow('acme-toolbar')).getByTestId('ui-enabled-chip')).toHaveTextContent('Enabled'));
    expect(api.calls).toEqual(['approve:acme-toolbar', 'disable:acme-toolbar', 'enable:acme-toolbar']);
  });

  it('grants a declared capability, with the hosts listed for the host allowlist; delete asks first', async () => {
    const { api } = setup([row({ approved: true, enabled: true })], ['acme-toolbar']);
    const card = await waitFor(() => screen.getByTestId('ui-plugin-card'));
    fireEvent.click(within(card).getByRole('button', { name: 'Grant ui:message-toolbar to Acme toolbar' }));
    await waitFor(() => expect(screen.getByTestId('ui-plugins-notice')).toHaveTextContent('Granted ui:message-toolbar.'));
    const hosts = within(card).getByRole('textbox', { name: 'Hosts for net:host-allowlist on Acme toolbar' });
    fireEvent.input(hosts, { target: { value: 'api.acme.example, cdn.acme.example' } });
    fireEvent.keyDown(hosts, { key: 'Enter' });
    fireEvent.click(within(card).getByRole('button', { name: 'Grant net:host-allowlist to Acme toolbar' }));
    await waitFor(() =>
      expect(api.calls).toEqual(['grant:acme-toolbar:ui:message-toolbar:{}', 'grant:acme-toolbar:net:host-allowlist:{"hosts":["api.acme.example","cdn.acme.example"]}']),
    );
    fireEvent.click(within(card).getByRole('button', { name: /^Delete/ }));
    const dialog = await screen.findByRole('alertdialog');
    expect(api.calls).not.toContain('remove:acme-toolbar');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(screen.queryByTestId('ui-plugin-card')).toBeNull());
    expect(api.calls.at(-1)).toBe('remove:acme-toolbar');
  });

  it('has no axe violations and no inline style', async () => {
    const { container } = setup([row()], ['acme-toolbar']);
    await waitFor(() => screen.getByTestId('ui-plugin-card'));
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });

  it('encodes a bundle file as the base64 the register route decodes', async () => {
    expect(await fileToBase64(new Blob([new Uint8Array([0, 1, 2, 250, 255])]))).toBe('AAEC+v8=');
  });

  it('reads manifests as the server does and knows its own origin', () => {
    expect(readManifest(JSON.stringify(MANIFEST)).parsed?.signed).toBe(false);
    expect(stripIsolates(readManifest('{"id":"x","name":"y"}').problem ?? '')).toMatch(/no version/);
    expect(sameOrigin('/plugins/x.json', 'https://mail.example/admin')).toBe(true);
    expect(sameOrigin('https://other.example/x.json', 'https://mail.example/admin')).toBe(false);
  });
});
