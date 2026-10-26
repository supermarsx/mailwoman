import { describe, it, expect } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { AdminPlugins, REGISTER, parseHosts } from './index.tsx';
import { registerInput } from './RegisterPlugin.tsx';
import { anyUnsignedLoaded, createPluginsSlice, type PluginInfo, type PluginsSlice } from '../../../state/slices/plugins.ts';
import { mockPluginsApi, pluginInfo, thirdPartyInfo } from './testkit.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { sectionPath, type AdminRoute } from '../shell/router.ts';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';
import { stripIsolates } from '../../../i18n';

function setup(plugins: PluginInfo[] = [], rest: string[] = [], slice?: PluginsSlice) {
  const api = mockPluginsApi(plugins);
  const [route, setRoute] = createSignal<AdminRoute>({ section: 'plugins', rest });
  const navigate = (path: string): void => {
    setRoute({ section: 'plugins', rest: (path.split('#/plugins')[1] ?? '').split('/').filter(Boolean).map(decodeURIComponent) });
  };
  const utils = renderInShell(() => <AdminPlugins {...(slice ? { slice } : { api })} route={route()} navigate={navigate} />);
  return { ...utils, api, setRoute, navigate };
}

/** The open plugin's detail. */
async function detail(): Promise<HTMLElement> {
  return waitFor(() => screen.getByTestId('plugin-card'));
}
const row = (id: string): HTMLElement => document.querySelector(`[data-row-key="${id}"]`) as HTMLElement;

describe('plugins slice', () => {
  it('raises the unsigned flag only for a loaded third-party plugin without a signature', () => {
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: true })])).toBe(true);
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: false, enabled: true })])).toBe(false);
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: true, signed: true })])).toBe(false);
    expect(anyUnsignedLoaded([pluginInfo({ loaded: true })])).toBe(false);
  });

  it('keeps a refused change as lastError and clears it on the next accepted one', async () => {
    const slice = createPluginsSlice(mockPluginsApi([thirdPartyInfo({ approved: true })]));
    await slice.load();
    expect(await slice.enable('acme-spam')).toBe(false);
    expect(slice.lastError()?.code).toBe('unsigned-not-allowed');
    expect(await slice.setAllowUnsigned('acme-spam', true)).toBe(true);
    expect(slice.lastError()).toBeNull();
  });

  it('splits a host list on commas and drops blanks', () => {
    expect(parseHosts(' rspamd.internal, ,10.0.0.5,')).toEqual(['rspamd.internal', '10.0.0.5']);
  });

  it('a first-party registration sends the id, and hosts only when given', () => {
    expect(registerInput({ component: 'spam-rspamd', hosts: [] })).toEqual({ id: 'spam-rspamd' });
    expect(registerInput({ component: 'spam-rspamd', hosts: ['scan.internal'] })).toEqual({ id: 'spam-rspamd', netAllowlist: ['scan.internal'] });
  });
});

describe('Admin → Plugins: list', () => {
  it('lists plugins with trust, running state and approval; the empty state offers registration', async () => {
    setup();
    expect(await screen.findByText('No plugins are registered.')).toBeInTheDocument();
    expect(within(within(screen.getByTestId('admin-plugins')).getByTestId('table-empty')).getByRole('button', { name: 'Register a plugin' })).toBeInTheDocument();
  });

  it('shows what the server says runs: loaded, not loaded, restart required', async () => {
    setup([
      pluginInfo({ approved: true, enabled: true, granted: ['spam-action'], loaded: true, loadedCapabilities: ['spam-action'], notLoadedReason: null }),
      thirdPartyInfo({ approved: true, enabled: true, restartRequired: true, notLoadedReason: null }),
    ]);
    await waitFor(() => expect(row('spam-rspamd')).not.toBeNull());
    expect(within(row('spam-rspamd')).getByTestId('status-chip')).toHaveTextContent('Loaded');
    expect(within(row('spam-rspamd')).getByTestId('trust-chip')).toHaveTextContent('First-party, digest built in');
    expect(within(row('spam-rspamd')).queryByTestId('sig-chip')).toBeNull();
    expect(within(row('acme-spam')).getByTestId('status-chip')).toHaveTextContent('Restart required');
    expect(within(row('acme-spam')).getByTestId('sig-chip')).toHaveTextContent('Unsigned');
  });

  it('approve and enable act at once; disable asks first, naming the plugin', async () => {
    const { api } = setup([pluginInfo()]);
    await waitFor(() => expect(row('spam-rspamd')).not.toBeNull());
    fireEvent.click(within(row('spam-rspamd')).getByRole('button', { name: /^Approve/ }));
    await waitFor(() => expect(within(row('spam-rspamd')).getByRole('button', { name: /^Enable/ })).toBeInTheDocument());
    fireEvent.click(within(row('spam-rspamd')).getByRole('button', { name: /^Enable/ }));
    await waitFor(() => expect(within(row('spam-rspamd')).getByRole('button', { name: /^Disable/ })).toBeInTheDocument());
    fireEvent.click(within(row('spam-rspamd')).getByRole('button', { name: /^Disable/ }));
    const dialog = await screen.findByRole('alertdialog');
    expect(stripIsolates(dialog.textContent ?? '')).toMatch(/Disable Rspamd spam classifier\?/);
    fireEvent.click(within(dialog).getByRole('button', { name: 'Disable' }));
    await waitFor(() => expect(api.calls).toEqual(['approve:spam-rspamd', 'enable:spam-rspamd', 'disable:spam-rspamd']));
    await waitFor(() => expect(stripIsolates(screen.getByTestId('plugins-notice').textContent ?? '')).toMatch(/is disabled/));
  });

  it("a refused enable shows the server's refusal", async () => {
    setup([thirdPartyInfo({ approved: true })]);
    await waitFor(() => expect(row('acme-spam')).not.toBeNull());
    fireEvent.click(within(row('acme-spam')).getByRole('button', { name: /^Enable/ }));
    expect(await screen.findByTestId('plugins-error')).toHaveTextContent('This plugin has no signature. Allow it to run unsigned before enabling it.');
  });

  it('chips filter third-party and not-running plugins', async () => {
    setup([pluginInfo(), thirdPartyInfo()]);
    await waitFor(() => expect(row('spam-rspamd')).not.toBeNull());
    fireEvent.click(screen.getByRole('button', { name: 'Third-party only', pressed: false }));
    await waitFor(() => expect(row('spam-rspamd')).toBeNull());
    expect(row('acme-spam')).not.toBeNull();
  });
});

describe('Admin → Plugins: registration', () => {
  it('registers a first-party component by id with no field changed, then opens it', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('plugin-register');
    expect(within(form).getByRole('combobox', { name: 'Component' })).toHaveValue('bridge-graph');
    fireEvent.change(within(form).getByRole('combobox', { name: 'Component' }), { target: { value: 'spam-rspamd' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    const d = await detail();
    expect(api.calls).toEqual(['register:{"id":"spam-rspamd"}']);
    expect(within(d).getByTestId('status-line')).toHaveTextContent('Not loaded: an administrator has not approved this plugin.');
  });

  it('sends the hosts typed for a first-party component', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('plugin-register');
    fireEvent.change(within(form).getByRole('combobox', { name: 'Component' }), { target: { value: 'spam-rspamd' } });
    const hosts = within(form).getByRole('textbox', { name: 'Hosts the plugin may contact' });
    fireEvent.input(hosts, { target: { value: 'scan.internal, 10.0.0.5' } });
    fireEvent.keyDown(hosts, { key: 'Enter' });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await detail();
    expect(api.calls).toEqual(['register:{"id":"spam-rspamd","netAllowlist":["scan.internal","10.0.0.5"]}']);
  });

  it('registers a third-party component with its manifest and never offers account-backend', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('plugin-register');
    fireEvent.change(within(form).getByRole('combobox', { name: 'Component' }), { target: { value: '' } });
    expect(within(form).queryByText('account-backend')).toBeNull();
    fireEvent.input(within(form).getByRole('textbox', { name: 'Plugin id' }), { target: { value: 'acme-spam' } });
    fireEvent.input(within(form).getByRole('textbox', { name: 'Name' }), { target: { value: 'Acme spam filter' } });
    fireEvent.input(within(form).getByRole('textbox', { name: 'Version' }), { target: { value: '1.0.0' } });
    fireEvent.click(within(form).getByRole('checkbox', { name: 'spam-action' }));
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    const d = await detail();
    expect(api.calls).toEqual(['register:{"id":"acme-spam","name":"Acme spam filter","version":"1.0.0","capabilities":["spam-action"],"netAllowlist":[]}']);
    expect(within(d).getByTestId('trust-chip')).toHaveTextContent('Third-party, digest approved');
  });

  it('a third-party registration needs id, name and version before any request', async () => {
    const { api } = setup([], [REGISTER]);
    const form = await screen.findByTestId('plugin-register');
    fireEvent.change(within(form).getByRole('combobox', { name: 'Component' }), { target: { value: '' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(within(form).getAllByText('Enter a value.').length).toBe(3));
    expect(api.calls).toEqual([]);
  });

  it('shows the refusal at the id when the server will not register', async () => {
    const { api } = setup([thirdPartyInfo()], [REGISTER]);
    const form = await screen.findByTestId('plugin-register');
    fireEvent.change(within(form).getByRole('combobox', { name: 'Component' }), { target: { value: '' } });
    fireEvent.input(within(form).getByRole('textbox', { name: 'Plugin id' }), { target: { value: 'acme-spam' } });
    fireEvent.input(within(form).getByRole('textbox', { name: 'Name' }), { target: { value: 'Again' } });
    fireEvent.input(within(form).getByRole('textbox', { name: 'Version' }), { target: { value: '2' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Register' }));
    await waitFor(() => expect(within(form).getByText('This plugin is already registered. Uninstall it before registering it again.')).toBeInTheDocument());
    expect(within(form).getByRole('textbox', { name: 'Plugin id' })).toHaveAttribute('aria-invalid', 'true');
    expect(api.calls.length).toBe(1);
  });
});

describe('Admin → Plugins: detail', () => {
  it('grant: loaded only once the grant exists, with exactly that grant', async () => {
    const { api } = setup([pluginInfo({ approved: true, enabled: true, notLoadedReason: 'no-grant' })], ['spam-rspamd']);
    const d = await detail();
    expect(within(d).getByTestId('status-line')).toHaveTextContent('no capability it needs has been granted');
    const save = within(d).getByRole('button', { name: 'Save grants' });
    expect(save).toBeDisabled();
    fireEvent.click(within(d).getByRole('checkbox', { name: 'spam-action' }));
    fireEvent.click(save);
    await waitFor(() => expect(within(d).getByTestId('running-with')).toHaveTextContent('Running with: spam-action'));
    expect(api.calls).toEqual(['grant:spam-rspamd:{"accountId":null,"capabilities":["spam-action"]}']);
  });

  it('saves the classifier address and shows the test verdict the component returned', async () => {
    const { api } = setup(
      [pluginInfo({ approved: true, enabled: true, granted: ['spam-action', 'net'], loaded: true, loadedCapabilities: ['net', 'spam-action'], notLoadedReason: null })],
      ['spam-rspamd'],
    );
    const d = await detail();
    fireEvent.input(within(d).getByRole('textbox', { name: 'Classifier address' }), { target: { value: 'http://rspamd:11333' } });
    fireEvent.click(within(d).getByRole('button', { name: 'Save address' }));
    await waitFor(() => expect(api.calls).toContain('endpoint:spam-rspamd:http://rspamd:11333'));
    fireEvent.click(within(d).getByRole('button', { name: 'Test classifier' }));
    const result = await within(d).findByTestId('test-result');
    expect(result).toHaveTextContent('Test verdict: spam');
    expect(result).toHaveTextContent('"score": 15');
  });

  it('the test button is unavailable while the plugin is not loaded, and says why', async () => {
    setup([pluginInfo()], ['spam-rspamd']);
    const d = await detail();
    const test = within(d).getByRole('button', { name: 'Test classifier' });
    expect(test).toBeDisabled();
    expect(test).toHaveAccessibleDescription('The plugin is not loaded, so it cannot be tested.');
  });

  it('allowing an unsigned third-party plugin asks first; the banner follows a loaded one', async () => {
    const { api, navigate } = setup([thirdPartyInfo({ approved: true, granted: ['spam-action'] })], ['acme-spam']);
    const d = await detail();
    const allow = within(d).getByRole('checkbox', { name: 'Allow this unsigned plugin to run' });
    fireEvent.click(allow);
    expect(allow).not.toBeChecked();
    const dialog = await screen.findByRole('alertdialog');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Allow unsigned' }));
    await waitFor(() => expect(api.calls).toEqual(['allow:acme-spam:true']));
    await waitFor(() => expect(within(d).getByRole('checkbox', { name: 'Allow this unsigned plugin to run' })).toBeChecked());
    navigate(sectionPath('plugins'));
    await waitFor(() => expect(row('acme-spam')).not.toBeNull());
    fireEvent.click(within(row('acme-spam')).getByRole('button', { name: /^Enable/ }));
    navigate(sectionPath('plugins', ['acme-spam']));
    expect(await screen.findByTestId('unsigned-banner')).toBeInTheDocument();
  });

  it('a first-party component has no signature chip, no allow-unsigned control and no banner', async () => {
    setup([pluginInfo({ approved: true, enabled: true, granted: ['spam-action'], loaded: true, notLoadedReason: null })], ['spam-rspamd']);
    const d = await detail();
    expect(within(d).queryByTestId('sig-chip')).toBeNull();
    expect(within(d).queryByTestId('allow-unsigned')).toBeNull();
    expect(screen.queryByTestId('unsigned-banner')).toBeNull();
  });

  it('uninstall asks for the id to be typed, then removes the plugin and returns to the list', async () => {
    const { api } = setup([pluginInfo()], ['spam-rspamd']);
    const d = await detail();
    fireEvent.click(within(d).getByRole('button', { name: /^Uninstall/ }));
    const dialog = await screen.findByRole('alertdialog');
    const confirm = within(dialog).getByRole('button', { name: 'Uninstall' });
    expect(confirm).toBeDisabled();
    fireEvent.input(within(dialog).getByRole('textbox'), { target: { value: 'spam-rspamd' } });
    fireEvent.click(confirm);
    await waitFor(() => expect(api.calls).toEqual(['uninstall:spam-rspamd']));
    await waitFor(() => expect(screen.queryByTestId('plugin-card')).toBeNull());
  });

  it('has no axe violations and no inline style (list + detail)', async () => {
    const { container } = setup([pluginInfo({ approved: true }), thirdPartyInfo()], ['acme-spam']);
    await detail();
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });

  it('an unknown id says so', async () => {
    setup([pluginInfo()], ['nope']);
    await waitFor(() => expect(stripIsolates(screen.getByRole('alert').textContent ?? '')).toMatch(/There is no plugin with the id nope/));
    expect(sectionPath('plugins', [REGISTER])).toContain('%2Bregister');
  });
});
