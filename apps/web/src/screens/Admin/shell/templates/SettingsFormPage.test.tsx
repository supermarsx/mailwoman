import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { SettingsFormPage, validateSettingField, type SettingsGroup, type SettingsLoad } from './SettingsFormPage.tsx';
import { toasts } from '../../../../components/toast/index.ts';
import { anyDirty } from '../useDirtyGuard.ts';
import { renderInShell } from '../testkit.tsx';
import { stripIsolates } from '../../../../i18n';
import { expectNoAxeViolations, expectNoInlineStyle, problem } from '../uxTestkit.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

const GROUPS: SettingsGroup[] = [
  {
    id: 'smtp',
    heading: 'Outgoing mail',
    help: 'How the server hands mail to the relay.',
    fields: [
      {
        id: 'smtp.host',
        label: 'Relay host',
        help: 'The host name of the relay; not used for incoming mail.',
        docsHref: 'docs/admin/mail.md#smtp-host',
        kind: 'text',
        required: true,
        source: { kind: 'stored' },
        propagation: 'hot',
        history: { target: 'setting:smtp.host', updatedBy: 'alice', updatedAt: '2026-10-20T08:00:00Z' },
      },
      {
        id: 'smtp.port',
        label: 'Port',
        help: 'The relay port.',
        kind: 'number',
        min: 1,
        max: 65535,
        step: 1,
        source: { kind: 'default' },
        propagation: 'restart',
      },
      {
        id: 'smtp.tls',
        label: 'Use TLS',
        help: 'Encrypt the connection to the relay.',
        kind: 'boolean',
      },
    ],
  },
  {
    id: 'env',
    heading: 'Fixed by the environment',
    fields: [
      {
        id: 'log.level',
        label: 'Log level',
        technical: 'MW_LOG_LEVEL',
        help: 'How much the server logs.',
        kind: 'select',
        options: [
          { value: 'info', label: 'Info' },
          { value: 'debug', label: 'Debug' },
        ],
        source: { kind: 'env-pin', envVar: 'MW_LOG_LEVEL' },
      },
      {
        id: 'relay.password',
        label: 'Relay password',
        help: 'Write-only; the stored value is never shown.',
        kind: 'secret',
      },
    ],
  },
];

const VALUES = { 'smtp.host': 'relay.example.org', 'smtp.port': 587, 'smtp.tls': true, 'log.level': 'info', 'relay.password': { set: true } };

function setup(opts: { load?: () => Promise<SettingsLoad>; save?: (...a: unknown[]) => Promise<unknown>; groups?: SettingsGroup[] } = {}) {
  const load = vi.fn(opts.load ?? (async () => ({ values: { ...VALUES }, etag: '"v1"' })));
  const save = vi.fn(opts.save ?? (async () => ({ etag: '"v2"' })));
  const utils = renderInShell(() => (
    <SettingsFormPage
      title="Mail relay"
      description="Settings for outgoing mail."
      groups={opts.groups ?? GROUPS}
      load={load}
      save={save as never}
      unavailableReason="the relay settings arrive with a later server version."
    />
  ));
  return { ...utils, load, save };
}

const field = (label: string | RegExp) => screen.getByLabelText(label, { selector: 'input, select, textarea' });

describe('SettingsFormPage', () => {
  it('fieldsets with headings; each field has label, help with Learn more, source badge, propagation and history', async () => {
    setup();
    const group = await screen.findByRole('group', { name: 'Outgoing mail' });
    const host = within(group).getByRole('textbox', { name: 'Relay host' });
    expect(host).toHaveValue('relay.example.org');
    expect(host).toHaveAccessibleDescription(/The host name of the relay; not used for incoming mail\. Learn more/);
    expect(within(group).getByRole('link', { name: 'Learn more about Relay host' })).toHaveAttribute('href', 'docs/admin/mail.md#smtp-host');
    const block = screen.getByTestId('setting-smtp.host');
    expect(block).toHaveAttribute('data-field-anchor', 'smtp.host');
    expect(within(block).getByTestId('source-badge')).toHaveTextContent('Stored');
    expect(within(block).getByText('Applies within 30 seconds on every server process.')).toBeInTheDocument();
    expect(stripIsolates(within(block).getByTestId('field-history').textContent ?? '')).toMatch(/Last changed by alice/);
    expect(within(screen.getByTestId('setting-smtp.port')).getByText('Applies after the server restarts.')).toBeInTheDocument();
  });

  it('a value pinned by the environment is disabled and its reason is read with it', async () => {
    setup();
    const level = await screen.findByRole('combobox', { name: /Log level/ });
    expect(level).toBeDisabled();
    expect(stripIsolates(level.getAttribute('aria-label') ?? '')).toBe('');
    expect(level).toHaveAccessibleDescription(/The server environment sets this value with .*MW_LOG_LEVEL.*\. Change it there and restart the server\./);
  });

  it('the save bar counts changed fields; Discard puts the stored values back', async () => {
    setup();
    const bar = await screen.findByTestId('save-bar');
    expect(bar).toHaveTextContent('No unsaved changes');
    expect(within(bar).getByRole('button', { name: 'Save' })).toBeDisabled();
    fireEvent.input(field('Relay host'), { target: { value: 'relay2.example.org' } });
    fireEvent.input(field('Port'), { target: { value: '2525' } });
    expect(bar).toHaveTextContent('2 unsaved changes');
    expect(anyDirty()).toBe(true);
    fireEvent.click(within(bar).getByRole('button', { name: 'Discard changes' }));
    expect(bar).toHaveTextContent('No unsaved changes');
    expect(field('Relay host')).toHaveValue('relay.example.org');
    expect(anyDirty()).toBe(false);
  });

  it('Save validates every field first: no request, errors at the fields, focus on the first', async () => {
    const { save } = setup();
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Relay host'), { target: { value: '' } });
    fireEvent.input(field('Port'), { target: { value: '70000' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(document.activeElement).toBe(field('Relay host')));
    expect(save).not.toHaveBeenCalled();
    expect(field('Relay host')).toHaveAttribute('aria-invalid', 'true');
    expect(field('Relay host')).toHaveAccessibleDescription(/Enter a value\./);
    expect(field('Port')).toHaveAccessibleDescription(/Enter 65,535 or less\./);
    expect(screen.getByTestId('save-result')).toHaveTextContent('Nothing was saved. Fix the 2 fields marked below.');
  });

  it('shows no initial success, keeps Save readable while pending, and replaces progress with one success toast', async () => {
    let finish!: () => void;
    setup({save: () => new Promise<void>(resolve => {finish=resolve;})});
    await screen.findByTestId('save-bar');
    expect(screen.queryByTestId('admin-form-save-toast')).toBeNull();
    fireEvent.input(field('Port'), {target:{value:'2525'}});
    const save=screen.getByRole('button',{name:'Save'});
    fireEvent.click(save);
    expect(save).toBeDisabled();
    expect(save).toHaveAttribute('aria-busy','true');
    expect(screen.getByTestId('admin-form-save-toast')).toHaveTextContent('Saving');
    expect(toasts().find(item=>item.testId==='admin-form-save-toast')).toMatchObject({busy:true,persistent:true,duration:Infinity});
    expect(screen.getByTestId('save-result')).toBeEmptyDOMElement();
    finish();
    await waitFor(()=>expect(screen.getByTestId('admin-form-save-toast')).toHaveTextContent('Saved 1 change.'));
    expect(screen.getAllByTestId('admin-form-save-toast')).toHaveLength(1);
    expect(screen.getByTestId('save-result')).toBeEmptyDOMElement();
    expect(save).toHaveAttribute('aria-busy','false');
    expect(toasts().find(item=>item.testId==='admin-form-save-toast')).toMatchObject({busy:false,persistent:false});
  });

  it('sends only the changed fields with If-Match, then announces the result', async () => {
    const { save } = setup();
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Port'), { target: { value: '2525' } });
    fireEvent.click(field('Use TLS'));
    fireEvent.input(field('Relay password'), { target: { value: 'n3w' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect(save.mock.calls[0]![0]).toEqual({ 'smtp.port': 2525, 'smtp.tls': false, 'relay.password': 'n3w' });
    expect((save.mock.calls[0]![1] as { ifMatch: string }).ifMatch).toBe('"v1"');
    await waitFor(() => expect(screen.getByTestId('admin-form-save-toast')).toHaveTextContent('Saved 3 changes.'));
    expect(screen.getByTestId('save-bar')).toHaveTextContent('No unsaved changes');
    expect(field('Relay password')).toHaveValue('');
  });

  it("a 422 with a JSON pointer shows the server's error at that field and focuses it", async () => {
    setup({ save: async () => Promise.reject(problem(422, { code: 'out_of_range', field: '/smtp/port' })) });
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Port'), { target: { value: '25' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(document.activeElement).toBe(field('Port')));
    expect(field('Port')).toHaveAccessibleDescription(/outside the allowed range/);
    expect(screen.queryByTestId('admin-form-save-toast')).toBeNull();
    expect(screen.getByTestId('save-bar')).toHaveTextContent('1 unsaved change');
  });

  it('a 412 shows who changed it; Keep mine saves again with the current tag; Reload reads again', async () => {
    const tags: (string | null)[] = [];
    const { load } = setup({
      save: async (_changed: unknown, ctx: unknown) => {
        const { ifMatch } = ctx as { ifMatch: string | null };
        tags.push(ifMatch);
        if (ifMatch === '"v1"') throw problem(412, { code: 'stale', updatedBy: 'bob', updatedAt: '2026-10-25T10:00:00Z' }, '"v5"');
        return { etag: '"v6"' };
      },
    });
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Port'), { target: { value: '2525' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    const notice = await screen.findByTestId('stale-notice');
    expect(stripIsolates(notice.textContent ?? '')).toMatch(/^bob changed this on /);
    expect(screen.getByTestId('save-bar')).toHaveTextContent('1 unsaved change');
    fireEvent.click(within(notice).getByRole('button', { name: 'Keep mine' }));
    await waitFor(() => expect(screen.queryByTestId('stale-notice')).toBeNull());
    expect(tags).toEqual(['"v1"', '"v5"']);
    await waitFor(() => expect(screen.getByTestId('save-bar')).toHaveTextContent('No unsaved changes'));
    expect(screen.getByTestId('admin-form-save-toast')).toHaveTextContent('Saved 1 change.');
    expect(load).toHaveBeenCalledTimes(1);
  });

  it('Reload after a 412 reads the values again and drops the edits', async () => {
    let first = true;
    const { load } = setup({
      save: async () => {
        if (first) {
          first = false;
          throw problem(412, { code: 'stale', updatedBy: 'bob' });
        }
        return {};
      },
    });
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Port'), { target: { value: '2525' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    fireEvent.click(within(await screen.findByTestId('stale-notice')).getByRole('button', { name: 'Reload' }));
    await waitFor(() => expect(load).toHaveBeenCalledTimes(2));
    expect(await screen.findByLabelText('Port', { selector: 'input' })).toHaveValue(587);
  });

  it('states: loading, error with code and Try again, and not available', async () => {
    let fail = true;
    setup({
      load: async () => {
        if (fail) {
          fail = false;
          throw problem(500, { code: 'db_unavailable' });
        }
        return { values: VALUES };
      },
    });
    expect(screen.getByRole('status')).toHaveTextContent('Loading');
    expect(await screen.findByText('Error code: db_unavailable')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByRole('group', { name: 'Outgoing mail' })).toBeInTheDocument();
  });

  it('a server without the route shows Not available with the reason', async () => {
    setup({ load: async () => 'unavailable' });
    expect(await screen.findByText('Not available — the relay settings arrive with a later server version.')).toBeInTheDocument();
    expect(screen.queryByTestId('save-bar')).toBeNull();
  });

  it('Revert to default runs the field action', async () => {
    const onRevert = vi.fn(async () => undefined);
    const groups: SettingsGroup[] = [
      { id: 'g', heading: 'G', fields: [{ ...GROUPS[0]!.fields[1]!, revert: { defaultDisplay: '25', isDefault: false, onRevert } }] },
    ];
    setup({ groups });
    fireEvent.click(await screen.findByRole('button', { name: 'Revert to default (Port)' }));
    await waitFor(() => expect(onRevert).toHaveBeenCalledTimes(1));
  });

  it('keyboard: the checkbox has its label beside it and toggles with Space; the form submits with Enter', async () => {
    const { save } = setup();
    await screen.findByTestId('save-bar');
    const tls = field('Use TLS');
    expect(tls.nextElementSibling?.tagName).toBe('LABEL');
    fireEvent.click(tls);
    fireEvent.submit(tls.closest('form')!);
    await waitFor(() => expect(save).toHaveBeenCalledWith({ 'smtp.tls': false }, expect.anything()));
  });

  it('has no axe violations and no inline style attributes, with errors shown', async () => {
    const { container } = setup();
    await screen.findByTestId('save-bar');
    fireEvent.input(field('Relay host'), { target: { value: '' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await screen.findByText('Enter a value.');
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});

describe('validateSettingField', () => {
  const f = (over: object) => ({ id: 'x', label: 'X', help: 'h', kind: 'text' as const, ...over });
  it('checks type and declared bounds', () => {
    expect(validateSettingField(f({ kind: 'number', min: 1 }), 'abc')).toBe('Enter a number.');
    expect(validateSettingField(f({ kind: 'number', step: 1 }), 1.5)).toBe('Enter a whole number.');
    expect(validateSettingField(f({ kind: 'number', min: 10 }), 3)).toBe('Enter 10 or more.');
    expect(validateSettingField(f({ maxLength: 3 }), 'abcd')).toBe('Use at most 3 characters.');
    expect(validateSettingField(f({ kind: 'url' }), 'ftp://x')).toMatch(/https:\/\//);
    expect(validateSettingField(f({ kind: 'url' }), 'https://x.example')).toBeNull();
    expect(validateSettingField(f({ kind: 'email' }), 'nope')).toMatch(/email address/);
    expect(validateSettingField(f({ pattern: '[a-z]+', patternHint: 'Lowercase letters only.' }), 'AB')).toBe('Lowercase letters only.');
    expect(validateSettingField(f({ kind: 'select', options: [{ value: 'a', label: 'A' }] }), 'b')).toBe('Choose one of the listed options.');
    expect(validateSettingField(f({ required: true }), '')).toBe('Enter a value.');
    expect(validateSettingField(f({}), '')).toBeNull();
    expect(validateSettingField(f({ kind: 'secret', required: true }), '')).toBeNull();
    expect(validateSettingField(f({ validate: () => 'Custom.' }), 'v')).toBe('Custom.');
  });
});
