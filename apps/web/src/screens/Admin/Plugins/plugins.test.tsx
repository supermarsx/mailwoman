import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminPlugins, parseHosts } from './index.tsx';
import { anyUnsignedLoaded, createPluginsSlice } from '../../../state/slices/plugins.ts';
import { mockPluginsApi, pluginInfo, thirdPartyInfo } from './testkit.ts';

/** The one plugin card on screen. */
async function card(): Promise<HTMLElement> {
  return waitFor(() => screen.getByTestId('plugin-card'));
}

describe('plugins slice', () => {
  it('raises the unsigned flag only for a loaded third-party plugin without a signature', () => {
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: true })])).toBe(true);
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: false, enabled: true })])).toBe(false);
    expect(anyUnsignedLoaded([thirdPartyInfo({ loaded: true, signed: true })])).toBe(false);
    // A first-party component is trusted by its digest, signature or not.
    expect(anyUnsignedLoaded([pluginInfo({ loaded: true })])).toBe(false);
  });

  it('keeps a refused change as lastError and clears it on the next accepted one', async () => {
    const api = mockPluginsApi([thirdPartyInfo({ approved: true })]);
    const slice = createPluginsSlice(api);
    await slice.load();

    expect(await slice.enable('acme-spam')).toBe(false);
    expect(slice.lastError()?.code).toBe('unsigned-not-allowed');
    expect(slice.plugins()[0]?.enabled).toBe(false);

    expect(await slice.setAllowUnsigned('acme-spam', true)).toBe(true);
    expect(slice.lastError()).toBeNull();
    expect(await slice.enable('acme-spam')).toBe(true);
    expect(slice.plugins()[0]?.enabled).toBe(true);
  });

  it('splits a host list on commas and drops blanks', () => {
    expect(parseHosts(' rspamd.internal, ,10.0.0.5,')).toEqual(['rspamd.internal', '10.0.0.5']);
    expect(parseHosts('   ')).toEqual([]);
  });
});

describe('Admin → Plugins: registration', () => {
  it('starts empty and registers a first-party component by id', async () => {
    const api = mockPluginsApi();
    render(() => <AdminPlugins api={api} />);
    await waitFor(() => expect(screen.getByTestId('plugins-empty')).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText('Component'), { target: { value: 'spam-rspamd' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Register a plugin' }));

    const c = await card();
    // Only the id is sent: the server refuses any other manifest key for these ids.
    expect(api.calls).toEqual(['register:{"id":"spam-rspamd"}']);
    expect(within(c).getByTestId('trust-chip')).toHaveTextContent('First-party, digest built in');
    expect(within(c).getByTestId('status-chip')).toHaveTextContent('Not loaded');
    expect(within(c).getByTestId('status-line')).toHaveTextContent(
      'Not loaded: an administrator has not approved this plugin.',
    );
    // Once registered it is no longer offered.
    expect(screen.queryByRole('option', { name: 'spam-rspamd' })).toBeNull();
  });

  it('sends the hosts an administrator typed for a first-party component', async () => {
    const api = mockPluginsApi();
    render(() => <AdminPlugins api={api} />);
    await waitFor(() => expect(screen.getByTestId('plugins-empty')).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText('Component'), { target: { value: 'spam-rspamd' } });
    fireEvent.input(screen.getByLabelText(/Hosts the plugin may contact/), {
      target: { value: 'scan.internal, 10.0.0.5' },
    });
    fireEvent.submit(screen.getByRole('form', { name: 'Register a plugin' }));
    await card();
    expect(api.calls).toEqual(['register:{"id":"spam-rspamd","netAllowlist":["scan.internal","10.0.0.5"]}']);
  });

  it('registers a third-party component with its manifest and never offers account-backend', async () => {
    const api = mockPluginsApi();
    render(() => <AdminPlugins api={api} />);
    await waitFor(() => expect(screen.getByTestId('plugins-empty')).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText('Component'), { target: { value: '' } });
    const form = screen.getByRole('form', { name: 'Register a plugin' });
    expect(within(form).queryByText('account-backend')).toBeNull();
    fireEvent.input(within(form).getByLabelText(/Plugin id/), { target: { value: 'acme-spam' } });
    fireEvent.input(within(form).getByLabelText('Name'), { target: { value: 'Acme spam filter' } });
    fireEvent.input(within(form).getByLabelText('Version'), { target: { value: '1.0.0' } });
    fireEvent.click(within(form).getByLabelText('spam-action'));
    fireEvent.submit(form);

    const c = await card();
    expect(api.calls).toEqual([
      'register:{"id":"acme-spam","name":"Acme spam filter","version":"1.0.0","capabilities":["spam-action"],"netAllowlist":[]}',
    ]);
    expect(within(c).getByTestId('trust-chip')).toHaveTextContent('Third-party, digest approved');
    expect(within(c).getByTestId('sig-chip')).toHaveTextContent('Unsigned');
  });

  it('shows the refusal when the server will not register', async () => {
    const api = mockPluginsApi([pluginInfo()]);
    const slice = createPluginsSlice(api);
    render(() => <AdminPlugins slice={slice} />);
    await card();
    await slice.register({ id: 'spam-rspamd' });
    await waitFor(() =>
      expect(screen.getByTestId('plugins-error')).toHaveTextContent(
        'This plugin is already registered. Uninstall it before registering it again.',
      ),
    );
  });
});

describe('Admin → Plugins: what runs is what the server says runs', () => {
  it('approve, enable, grant: loaded only once the grant exists, with exactly that grant', async () => {
    const api = mockPluginsApi([pluginInfo()]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();

    fireEvent.click(within(c).getByRole('button', { name: 'Approve' }));
    await waitFor(() => expect(within(c).getByTestId('status-line')).toHaveTextContent('the plugin is not enabled'));
    fireEvent.click(within(c).getByRole('button', { name: 'Enable' }));
    // Enabled is not loaded: nothing has been granted.
    await waitFor(() =>
      expect(within(c).getByTestId('status-line')).toHaveTextContent(
        'Not loaded: no capability it needs has been granted.',
      ),
    );
    expect(within(c).getByTestId('status-chip')).toHaveTextContent('Not loaded');
    expect(within(c).queryByTestId('running-with')).toBeNull();

    fireEvent.click(within(c).getByLabelText('Grant spam-action to Rspamd spam classifier'));
    fireEvent.click(within(c).getByRole('button', { name: 'Save grants' }));
    await waitFor(() => expect(within(c).getByTestId('status-chip')).toHaveTextContent('Loaded'));
    expect(api.calls).toContain('grant:spam-rspamd:{"accountId":null,"capabilities":["spam-action"]}');
    expect(within(c).getByTestId('running-with')).toHaveTextContent('Running with: spam-action');
    expect(within(c).queryByTestId('status-line')).toBeNull();

    // Unticking a capability and saving revokes it: the whole set is sent.
    fireEvent.click(within(c).getByLabelText('Grant spam-action to Rspamd spam classifier'));
    fireEvent.click(within(c).getByRole('button', { name: 'Save grants' }));
    await waitFor(() => expect(within(c).getByTestId('status-chip')).toHaveTextContent('Not loaded'));
    expect(api.calls).toContain('grant:spam-rspamd:{"accountId":null,"capabilities":[]}');
  });

  it('an enabled plugin the server has not loaded is shown as needing a restart, not as running', async () => {
    const api = mockPluginsApi([
      pluginInfo({
        id: 'bridge-gmail',
        name: 'Gmail API bridge',
        role: 'account-backend',
        capabilities: ['account-backend', 'net'],
        granted: ['account-backend', 'net'],
        approved: true,
        enabled: true,
        loaded: false,
        restartRequired: true,
        notLoadedReason: null,
      }),
    ]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    expect(within(c).getByTestId('status-chip')).toHaveTextContent('Restart required');
    expect(within(c).getByTestId('status-line')).toHaveTextContent('Restart required to load this plugin.');
    expect(within(c).queryByTestId('running-with')).toBeNull();
  });

  it('a plugin whose hooks the server never calls says so', async () => {
    const api = mockPluginsApi([
      pluginInfo({
        id: 'languagetool',
        name: 'LanguageTool grammar',
        role: 'none',
        capabilities: ['dlp-detector', 'net'],
        approved: true,
        enabled: true,
        notLoadedReason: 'no-host-caller',
      }),
    ]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    expect(within(c).getByTestId('status-line')).toHaveTextContent(
      'Not loaded: this server version does not call the hooks this plugin provides.',
    );
    // No classifier controls for a plugin that is not a classifier.
    expect(within(c).queryByRole('button', { name: 'Test classifier' })).toBeNull();
  });
});

describe('Admin → Plugins: unsigned third-party plugins', () => {
  it('enable is refused until the allow-unsigned flag is set; the banner follows a loaded one', async () => {
    const api = mockPluginsApi([thirdPartyInfo({ approved: true, granted: ['spam-action'] })]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    expect(screen.queryByTestId('unsigned-banner')).toBeNull();

    fireEvent.click(within(c).getByRole('button', { name: 'Enable' }));
    await waitFor(() =>
      expect(screen.getByTestId('plugins-error')).toHaveTextContent(
        'This plugin has no signature. Allow it to run unsigned before enabling it.',
      ),
    );
    expect(within(c).getByTestId('status-chip')).toHaveTextContent('Not loaded');

    fireEvent.click(within(c).getByLabelText('Allow unsigned plugin Acme spam filter'));
    await waitFor(() => expect(api.calls).toContain('allow:acme-spam:true'));
    // The flag alone enables nothing.
    expect(within(c).getByRole('button', { name: 'Enable' })).toBeInTheDocument();
    expect(screen.queryByTestId('unsigned-banner')).toBeNull();

    fireEvent.click(within(c).getByRole('button', { name: 'Enable' }));
    await waitFor(() => expect(within(c).getByTestId('status-chip')).toHaveTextContent('Loaded'));
    expect(screen.getByTestId('unsigned-banner')).toHaveTextContent(
      'A third-party plugin without a signature is loaded.',
    );
  });

  it('a first-party component has no signature chip, no allow-unsigned control and no banner', async () => {
    const api = mockPluginsApi([
      pluginInfo({
        approved: true,
        enabled: true,
        granted: ['spam-action'],
        loaded: true,
        loadedCapabilities: ['spam-action'],
        notLoadedReason: null,
      }),
    ]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    expect(within(c).queryByTestId('sig-chip')).toBeNull();
    expect(within(c).queryByTestId('allow-unsigned')).toBeNull();
    expect(screen.queryByTestId('unsigned-banner')).toBeNull();
  });
});

describe('Admin → Plugins: classifier settings and test', () => {
  it('saves the classifier address and shows the test verdict the component returned', async () => {
    const api = mockPluginsApi([
      pluginInfo({
        approved: true,
        enabled: true,
        granted: ['spam-action'],
        loaded: true,
        loadedCapabilities: ['spam-action'],
        notLoadedReason: null,
      }),
    ]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();

    fireEvent.input(within(c).getByLabelText('Classifier address for Rspamd spam classifier'), {
      target: { value: ' http://scan.internal:11333 ' },
    });
    fireEvent.click(within(c).getByRole('button', { name: 'Save address' }));
    await waitFor(() => expect(api.calls).toContain('endpoint:spam-rspamd:http://scan.internal:11333'));

    fireEvent.click(within(c).getByRole('button', { name: 'Test classifier' }));
    const result = await waitFor(() => within(c).getByTestId('test-result'));
    expect(result).toHaveTextContent('Test verdict: unknown');
    expect(result).toHaveTextContent('net capability not granted');
  });

  it('the test button is unavailable while the plugin is not loaded', async () => {
    const api = mockPluginsApi([pluginInfo({ approved: true })]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    expect(within(c).getByRole('button', { name: 'Test classifier' })).toBeDisabled();
  });

  it('uninstall asks once more, then removes the plugin', async () => {
    const api = mockPluginsApi([pluginInfo()]);
    render(() => <AdminPlugins api={api} />);
    const c = await card();
    fireEvent.click(within(c).getByRole('button', { name: 'Uninstall' }));
    expect(api.calls).toEqual([]);
    fireEvent.click(within(c).getByRole('button', { name: 'Confirm uninstall' }));
    await waitFor(() => expect(screen.queryByTestId('plugin-card')).toBeNull());
    expect(api.calls).toEqual(['uninstall:spam-rspamd']);
  });
});
