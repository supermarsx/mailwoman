import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminUiPlugins } from './index.tsx';
import { fileToBase64, UiPluginsApiError, type UiPluginInfo, type UiPluginsApi } from './service.ts';

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

const MANIFEST = { id: 'acme-toolbar', name: 'Acme toolbar', version: '1.2.0', capabilities: ['ui:message-toolbar'] };

describe('Admin → UI plugins', () => {
  it('starts empty; an unsigned manifest is refused until the administrator allows it', async () => {
    const api = mockApi();
    render(() => <AdminUiPlugins api={api} />);
    await waitFor(() => expect(screen.getByTestId('ui-plugins-empty')).toBeInTheDocument());

    const form = screen.getByRole('form', { name: 'Register a UI plugin' });
    fireEvent.input(within(form).getByLabelText('Manifest (JSON)'), { target: { value: JSON.stringify(MANIFEST) } });
    fireEvent.submit(form);
    await waitFor(() =>
      expect(screen.getByTestId('ui-plugins-error')).toHaveTextContent(
        'The server refused the request: plugin is unsigned and allowUnsigned is not set',
      ),
    );
    expect(screen.queryByTestId('ui-plugin-card')).toBeNull();

    fireEvent.click(within(form).getByLabelText('Allow this plugin to register without a signature'));
    fireEvent.submit(form);
    const card = await waitFor(() => screen.getByTestId('ui-plugin-card'));
    expect(api.calls.at(-1)).toBe(`register:${JSON.stringify(MANIFEST)}:null:true`);
    expect(within(card).getByTestId('ui-sig-chip')).toHaveTextContent('Unsigned');
    expect(within(card).queryByTestId('ui-enabled-chip')).toBeNull();
  });

  it('a manifest that is not JSON never reaches the server', async () => {
    const api = mockApi();
    render(() => <AdminUiPlugins api={api} />);
    const form = await waitFor(() => screen.getByRole('form', { name: 'Register a UI plugin' }));
    fireEvent.input(within(form).getByLabelText('Manifest (JSON)'), { target: { value: '{ not json' } });
    fireEvent.submit(form);
    await waitFor(() =>
      expect(screen.getByTestId('ui-plugins-error')).toHaveTextContent('The manifest is not valid JSON.'),
    );
    expect(api.calls).toEqual([]);
  });

  it('approve enables; disable and enable follow the server; delete asks once more', async () => {
    const api = mockApi([row()]);
    render(() => <AdminUiPlugins api={api} />);
    const card = await waitFor(() => screen.getByTestId('ui-plugin-card'));

    fireEvent.click(within(card).getByRole('button', { name: 'Approve and enable' }));
    await waitFor(() => expect(within(card).getByTestId('ui-enabled-chip')).toBeInTheDocument());
    fireEvent.click(within(card).getByRole('button', { name: 'Disable' }));
    await waitFor(() => expect(within(card).queryByTestId('ui-enabled-chip')).toBeNull());
    fireEvent.click(within(card).getByRole('button', { name: 'Enable' }));
    await waitFor(() => expect(within(card).getByTestId('ui-enabled-chip')).toBeInTheDocument());

    fireEvent.click(within(card).getByRole('button', { name: 'Delete' }));
    expect(api.calls).not.toContain('remove:acme-toolbar');
    fireEvent.click(within(card).getByRole('button', { name: 'Confirm delete' }));
    await waitFor(() => expect(screen.queryByTestId('ui-plugin-card')).toBeNull());
    expect(api.calls).toEqual(['approve:acme-toolbar', 'disable:acme-toolbar', 'enable:acme-toolbar', 'remove:acme-toolbar']);
  });

  it('grants a declared capability, with the typed hosts for the host allowlist', async () => {
    const api = mockApi([row({ approved: true, enabled: true })]);
    render(() => <AdminUiPlugins api={api} />);
    const card = await waitFor(() => screen.getByTestId('ui-plugin-card'));

    fireEvent.click(within(card).getByRole('button', { name: 'Grant ui:message-toolbar to Acme toolbar' }));
    await waitFor(() =>
      expect(screen.getByTestId('ui-plugins-notice')).toHaveTextContent('Granted ui:message-toolbar.'),
    );
    fireEvent.input(within(card).getByLabelText('Hosts for net:host-allowlist on Acme toolbar'), {
      target: { value: 'api.acme.example, cdn.acme.example' },
    });
    fireEvent.click(within(card).getByRole('button', { name: 'Grant net:host-allowlist to Acme toolbar' }));
    await waitFor(() =>
      expect(api.calls).toEqual([
        'grant:acme-toolbar:ui:message-toolbar:{}',
        'grant:acme-toolbar:net:host-allowlist:{"hosts":["api.acme.example","cdn.acme.example"]}',
      ]),
    );
  });

  it('encodes a bundle file as the base64 the register route decodes', async () => {
    expect(await fileToBase64(new Blob([new Uint8Array([0, 1, 2, 250, 255])]))).toBe('AAEC+v8=');
  });
});
