// t39-e6: the passkey policy keys use the generic scoped editor: a select with
// the registry's values, a global save with If-Match, a domain value, a lock.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { SettingDetail } from './SettingDetail.tsx';
import { KNOWN_KEYS } from './keys.ts';
import type { AdminSetting } from '../../../api/admin/settings.ts';
import { t } from '../../../test/i18n.ts';

function json(status: number, body: unknown, etag?: string): Response {
  const headers: Record<string, string> = { 'content-type': 'application/json' };
  if (etag) headers['ETag'] = etag;
  return new Response(JSON.stringify(body), { status, headers });
}

interface Call { url: string; method: string; body: unknown; ifMatch: string | null }

function stubFetch(handler: (c: Call) => Response): Call[] {
  const calls: Call[] = [];
  vi.stubGlobal('fetch', vi.fn(async (input: string, init?: RequestInit) => {
    const headers = new Headers(init?.headers);
    const c: Call = { url: String(input), method: init?.method ?? 'GET', body: init?.body ? JSON.parse(String(init.body)) : undefined, ifMatch: headers.get('If-Match') };
    calls.push(c);
    return handler(c);
  }));
  return calls;
}

function passkeySetting(key: string, values: string[], value: string): AdminSetting {
  return {
    key, family: false, kind: { type: 'enum', values }, default: value, nullable: false,
    scopes: 'global+domain+admin_user', userOverride: false, category: 'signin', mapsTo: '', env: null,
    dedicatedRoute: null, scopeId: 'account', hot: true, ceiling: false, userVisible: false,
    globalValue: null, allowUserOverride: false, globalLocked: false, updatedBy: null, updatedAt: null,
    effective: { value, source: 'default' }, envState: null, overrides: {}, etag: '"e0"',
  };
}

const KEYS: Array<[string, string[], string, RegExp]> = [
  ['twofa.passkeyAttachment', ['any', 'platform', 'cross-platform'], 'any', /Passkey authenticator type/],
  ['twofa.passkeyUserVerification', ['required', 'preferred', 'discouraged'], 'preferred', /Passkey user verification/],
  ['twofa.passkeyResidentKey', ['required', 'preferred', 'discouraged'], 'discouraged', /Discoverable passkeys/],
  ['twofa.passkeyLegacy', ['allow', 'refuse'], 'allow', /Passkeys added before the current policy/],
  ['twofa.lastFactorRemoval', ['allow', 'requireRecoveryCodes', 'block'], 'allow', /Removing the last second factor/],
];

describe('Admin › Settings › passkey policy', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('every passkey policy key is known to the console', () => {
    for (const [key] of KEYS) expect(KNOWN_KEYS).toContain(key);
  });

  it.each(KEYS)('%s offers exactly its values, labelled, and saves the choice', async (key, values, dflt, label) => {
    const pick = values.find((v) => v !== dflt)!;
    const calls = stubFetch((c) => c.method === 'PUT'
      ? json(200, { ...passkeySetting(key, values, dflt), globalValue: pick, etag: '"e1"' }, '"e1"')
      : json(200, { values: [] }));
    render(() => <SettingDetail settingKey={key} load={async () => ({ setting: passkeySetting(key, values, dflt), etag: '"e0"' })} />);
    const select = await screen.findByRole('combobox', { name: label });
    expect(within(select).getAllByRole('option').map((o) => (o as HTMLOptionElement).value)).toEqual(values);
    fireEvent.change(select, { target: { value: pick } });
    fireEvent.click(screen.getByRole('button', { name: t('admin-settings-save') }));
    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    const put = calls.find((c) => c.method === 'PUT')!;
    expect(put.url).toMatch(new RegExp(`/admin/settings/${key.replace('.', '\\.')}$`));
    expect(put.body).toEqual({ globalValue: pick });
    expect(put.ifMatch).toBe('"e0"');
  });

  it('a domain can have its own attachment rule', async () => {
    const key = 'twofa.passkeyAttachment';
    const values = ['any', 'platform', 'cross-platform'];
    const rows: unknown[] = [];
    const calls = stubFetch((c) => {
      if (c.method === 'PUT' && c.url.includes('/domains/')) {
        const row = { scope: 'domain', scopeId: 'example.org', value: 'platform', locked: true, updatedBy: 'root', updatedAt: 1, etag: '"d1"' };
        rows.push(row);
        return json(200, row, '"d1"');
      }
      if (c.url.includes('scope=domain')) return json(200, { values: rows, nextCursor: null });
      return json(200, { values: [] });
    });
    render(() => <SettingDetail settingKey={key} load={async () => ({ setting: passkeySetting(key, values, 'any'), etag: '"e0"' })} />);
    const section = (await screen.findByRole('heading', { name: t('admin-settings-domains') })).closest('section')!;
    fireEvent.input(within(section).getByRole('textbox', { name: t('admin-settings-domain') }), { target: { value: 'example.org' } });
    fireEvent.change(within(section).getByRole('combobox', { name: t('admin-settings-domain-value') }), { target: { value: 'platform' } });
    fireEvent.click(within(section).getByRole('button', { name: t('admin-settings-domain-save') }));
    await waitFor(() => expect(calls.some((c) => c.method === 'PUT')).toBe(true));
    const put = calls.find((c) => c.method === 'PUT')!;
    expect(put.url).toMatch(/\/admin\/settings\/twofa\.passkeyAttachment\/domains\/example\.org$/);
    expect((put.body as { value: string }).value).toBe('platform');
  });
});
