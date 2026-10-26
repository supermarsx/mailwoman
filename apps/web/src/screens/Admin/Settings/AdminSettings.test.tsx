// Admin › Settings (worker f01): the list (search, categories, sources, the
// unavailable state) and one setting's page (pinned and dedicated keys, client
// checks before any request, If-Match on save, 412 and 422, the lock preview,
// domain values), keyboard reachable, axe-clean and without inline styles.

import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { AdminSettings } from './index.tsx';
import { SettingDetail } from './SettingDetail.tsx';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';
import { t } from '../../../test/i18n.ts';

function setting(over: Partial<AdminSetting> & { key: string }): AdminSetting {
  return {
    family: false,
    kind: { type: 'int', min: 0, max: 10240 },
    default: 250,
    nullable: false,
    scopes: 'global+domain+user',
    userOverride: true,
    category: 'offline',
    mapsTo: '',
    env: null,
    dedicatedRoute: null,
    scopeId: 'account',
    hot: true,
    ceiling: false,
    userVisible: true,
    globalValue: null,
    allowUserOverride: true,
    globalLocked: false,
    updatedBy: null,
    updatedAt: null,
    effective: { value: 250, source: 'default' },
    envState: null,
    overrides: {},
    etag: '"e0"',
    ...over,
  };
}

const BUDGET = setting({ key: 'offline.budgetMb' });
const LANG = setting({
  key: 'ui.language',
  category: 'interface',
  kind: { type: 'locale', values: ['browser', 'en', 'de'] },
  default: 'browser',
  effective: { value: 'de', source: 'global' },
  globalValue: 'de',
  updatedBy: 'root',
  updatedAt: 1_760_000_000,
});
const PINNED = setting({
  key: 'realtime.recheckSecs',
  category: 'realtime',
  scopes: 'global',
  userVisible: false,
  env: { var: 'MW_REALTIME_RECHECK_SECS', precedence: 'pin' },
  envState: { var: 'MW_REALTIME_RECHECK_SECS', precedence: 'pin', set: true, value: 45 },
  effective: { value: 45, source: 'envPin' },
});
const SIGNIN = setting({
  key: 'signin.newUsers',
  category: 'signin',
  scopes: 'global+domain',
  kind: { type: 'enum', values: ['open', 'known', 'provisioned', 'closed'] },
  default: 'open',
  dedicatedRoute: '/signin-policy',
  effective: { value: 'open', source: 'default' },
});

function json(status: number, body: unknown, etag?: string): Response {
  const headers: Record<string, string> = { 'content-type': 'application/json' };
  if (etag) headers['ETag'] = etag;
  return new Response(JSON.stringify(body), { status, headers });
}

interface Call {
  url: string;
  method: string;
  body: unknown;
  ifMatch: string | null;
}

function stubFetch(handler: (c: Call) => Response): Call[] {
  const calls: Call[] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (input: string, init?: RequestInit) => {
      const headers = new Headers(init?.headers);
      const c: Call = {
        url: String(input),
        method: init?.method ?? 'GET',
        body: init?.body ? JSON.parse(String(init.body)) : undefined,
        ifMatch: headers.get('If-Match'),
      };
      calls.push(c);
      return handler(c);
    }),
  );
  return calls;
}

const route = (rest: string[] = []) => ({ section: 'settings', rest });

describe('Admin › Settings list', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('lists keys with their name, value and source, searchable and by category', async () => {
    const { container } = render(() => (
      <AdminSettings route={route()} load={async () => ({ kind: 'ok', data: [BUDGET, LANG, PINNED], etag: null })} />
    ));
    const table = await screen.findByRole('table');
    expect(within(table).getByText(t('admin-settings-key-offline-budget-mb'))).toBeInTheDocument();
    expect(within(table).getAllByText('ui.language').length).toBeGreaterThan(0);
    const badges = within(table).getAllByTestId('source-badge').map((b) => b.getAttribute('data-source'));
    expect(badges).toEqual(['stored', 'default', 'env-pin']);

    fireEvent.input(screen.getByRole('searchbox', { name: t('admin-settings-search') }), { target: { value: 'recheck' } });
    await waitFor(() => expect(within(screen.getByRole('table')).getAllByRole('row')).toHaveLength(2));

    fireEvent.input(screen.getByRole('searchbox', { name: t('admin-settings-search') }), { target: { value: '' } });
    fireEvent.click(await screen.findByRole('button', { name: t('admin-settings-cat-interface') }));
    await waitFor(() => expect(within(screen.getByRole('table')).getAllByRole('row')).toHaveLength(2));
    expect(within(screen.getByRole('table')).getAllByText('ui.language').length).toBeGreaterThan(0);

    expectNoInlineStyle(container);
    await expectNoAxeViolations(container);
  });

  it('says Not available on a server without the settings model', async () => {
    render(() => <AdminSettings route={route()} load={async () => ({ kind: 'unavailable' })} />);
    expect(await screen.findByText(new RegExp(t('admin-settings-unavailable').slice(0, 20)))).toBeInTheDocument();
  });
});

describe('Admin › one setting', () => {
  beforeEach(() => localStorage.clear());
  afterEach(() => vi.unstubAllGlobals());

  it('a pinned key cannot be edited and says which variable pins it', async () => {
    stubFetch(() => json(200, { values: [] }));
    render(() => <SettingDetail settingKey={PINNED.key} load={async () => ({ setting: PINNED, etag: '"p"' })} />);
    const input = await screen.findByRole('spinbutton', { name: /Live update re-check/ });
    expect(input).toBeDisabled();
    expect(screen.getAllByText(/MW_REALTIME_RECHECK_SECS/).length).toBeGreaterThan(0);
    expect(screen.getByRole('button', { name: t('admin-settings-save') })).toBeDisabled();
  });

  it('a sign-in key is read-only with a link to its own section', async () => {
    stubFetch(() => json(200, { values: [] }));
    render(() => <SettingDetail settingKey={SIGNIN.key} load={async () => ({ setting: SIGNIN, etag: '"s"' })} />);
    const link = await screen.findByRole('link', { name: /signin-policy/ });
    expect(link.getAttribute('href')).toContain('#/signin-policy');
    expect(screen.getByRole('combobox', { name: /New users/ })).toBeDisabled();
    expect(screen.queryByText(t('admin-settings-domains'))).toBeNull();
  });

  it('requires typed consequence confirmation before enabling last-superadmin loss', async () => {
    const policy = setting({ key: 'admin.allowLastSuperadminLoss', category: 'admin', kind: { type: 'bool' }, default: false, effective: { value: false, source: 'default' }, scopes: 'global', userOverride: false, allowUserOverride: false });
    const calls = stubFetch(c => c.method === 'PUT' ? json(200, { ...policy, globalValue: true, etag: '"e1"' }, '"e1"') : json(200, { values: [] }));
    render(() => <SettingDetail settingKey={policy.key} load={async () => ({ setting: policy, etag: '"e0"' })} />);
    fireEvent.click(await screen.findByRole('checkbox', { name: /Allow losing the last Super admin/ }));
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    const dialog = screen.getByRole('alertdialog');
    expect(calls.filter(c => c.method === 'PUT')).toHaveLength(0);
    expect(within(dialog).getByRole('button', { name: 'Allow loss of access' })).toBeDisabled();
    fireEvent.input(within(dialog).getByRole('textbox'), { target: { value: 'ALLOW LAST SUPERADMIN LOSS' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Allow loss of access' }));
    await waitFor(() => expect(calls.find(c => c.method === 'PUT')?.body).toEqual({ globalValue: true, confirmLastSuperadminLoss: true }));
  });

  it('checks the value before any request, then saves with If-Match', async () => {
    const calls = stubFetch((c) => {
      if (c.method === 'PUT') {
        return json(200, { ...BUDGET, globalValue: 512, effective: { value: 512, source: 'global' }, etag: '"e1"' }, '"e1"');
      }
      return json(200, { values: [] });
    });
    render(() => <SettingDetail settingKey={BUDGET.key} load={async () => ({ setting: BUDGET, etag: '"e0"' })} />);
    const input = await screen.findByRole('spinbutton', { name: /Offline cache size/ });
    fireEvent.input(input, { target: { value: '99999' } });
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    expect(await screen.findByText(t('admin-ux-err-max', { max: 10240 }))).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'PUT')).toHaveLength(0);

    fireEvent.input(input, { target: { value: '512' } });
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    expect(await screen.findByText(t('admin-settings-saved'))).toBeInTheDocument();
    const put = calls.find((c) => c.method === 'PUT')!;
    expect(put.url).toMatch(/\/admin\/settings\/offline\.budgetMb$/);
    expect(put.body).toEqual({ globalValue: 512 });
    expect(put.ifMatch).toBe('"e0"');
  });

  it('a 412 shows who changed it, a 422 shows at the field', async () => {
    let n = 0;
    stubFetch((c) => {
      if (c.method !== 'PUT') return json(200, { values: [] });
      n += 1;
      return n === 1
        ? json(412, { code: 'stale', updatedBy: 'alice', updatedAt: '2026-10-25T10:00:00Z' })
        : json(422, { code: 'invalidValue', detail: 'must be between 0 and 10240', field: '/globalValue' });
    });
    render(() => <SettingDetail settingKey={BUDGET.key} load={async () => ({ setting: BUDGET, etag: '"e0"' })} />);
    const input = await screen.findByRole('spinbutton', { name: /Offline cache size/ });
    fireEvent.input(input, { target: { value: '300' } });
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    expect(await screen.findByText(/alice/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: t('admin-ux-stale-keep') }));
    await waitFor(() => expect(input).toHaveAttribute('aria-invalid', 'true'));
  });

  it('locking over narrower values previews the impact first', async () => {
    const withOverrides = { ...BUDGET, overrides: { user: 2 } };
    const calls = stubFetch((c) => {
      if (c.method === 'PUT' && (c.body as { dryRun?: boolean }).dryRun) {
        return json(200, { dryRun: true, impact: { userValues: { count: 2, names: ['acct-1', 'acct-2'] } } });
      }
      if (c.method === 'PUT') return json(200, { ...withOverrides, globalLocked: true, etag: '"e2"' }, '"e2"');
      return json(200, { values: [] });
    });
    render(() => <SettingDetail settingKey={BUDGET.key} load={async () => ({ setting: withOverrides, etag: '"e0"' })} />);
    fireEvent.click(await screen.findByRole('switch'));
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    const dialog = await screen.findByRole('alertdialog');
    expect(await within(dialog).findByText(/acct-1/)).toBeInTheDocument();
    expect(calls.filter((c) => c.method === 'PUT')).toHaveLength(1); // the dry run only
    fireEvent.click(within(dialog).getByRole('button', { name: t('admin-settings-lock-confirm') }));
    await waitFor(() => expect(calls.filter((c) => c.method === 'PUT')).toHaveLength(2));
    expect(calls[calls.length - 1]!.body).toEqual({ locked: true });
  });

  it('adds a domain value and lists it', async () => {
    const domainRows: unknown[] = [];
    const calls = stubFetch((c) => {
      if (c.method === 'PUT' && c.url.includes('/domains/')) {
        const row = { scope: 'domain', scopeId: 'example.org', value: 64, locked: false, updatedBy: 'root', updatedAt: 1, etag: '"d1"' };
        domainRows.push(row);
        return json(200, row, '"d1"');
      }
      if (c.url.includes('scope=domain')) return json(200, { values: domainRows, nextCursor: null });
      return json(200, { values: [] });
    });
    render(() => <SettingDetail settingKey={BUDGET.key} load={async () => ({ setting: BUDGET, etag: '"e0"' })} />);
    const section = (await screen.findByRole('heading', { name: t('admin-settings-domains') })).closest('section')!;
    fireEvent.input(within(section).getByRole('textbox', { name: t('admin-settings-domain') }), {
      target: { value: 'example.org' },
    });
    fireEvent.input(within(section).getByRole('spinbutton', { name: t('admin-settings-domain-value') }), {
      target: { value: '64' },
    });
    fireEvent.click(within(section).getByRole('button', { name: t('admin-settings-domain-save') }));
    await waitFor(() => expect(within(section).getAllByText('example.org').length).toBeGreaterThan(0));
    const put = calls.find((c) => c.method === 'PUT')!;
    expect(put.url).toMatch(/\/admin\/settings\/offline\.budgetMb\/domains\/example\.org$/);
    expect(put.body).toEqual({ value: 64, locked: false });
  });
});
