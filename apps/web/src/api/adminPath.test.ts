// t32-h15: the console is wherever the server served it (`MW_ADMIN_PATH`). Every
// admin API URL and console link follows the page's own address once the server
// has marked the page as the console; nothing is built from a literal `/admin`.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { adminUrl, consolePath, isConsoleLocation, isConsolePage } from './adminPath.ts';
import { createHttpAdminApi } from '../state/slices/admin.ts';
import { createHttpPluginsApi } from '../state/slices/plugins.ts';
import { createHttpMaintenanceApi, createHttpSearchIndexApi } from './maintenance.ts';
import { createHttpSsoAdminApi } from '../modules/sso/client.ts';
import { createHttpTwofaPolicyApi } from '../screens/Admin/twofaPolicy.ts';
import { createAdminTotpVerifier } from '../screens/Admin/shell/adminAuth.ts';
import { createAdminMeLoader } from '../screens/Admin/shell/can.ts';
import { createAdminSearch } from '../screens/Admin/shell/CommandPalette.tsx';
import { createDashboardLoader } from './admin/dashboard.ts';
import { AdminAssistApi } from '../screens/Admin/Assist/service.ts';
import { createHttpUiPluginsApi } from '../screens/Admin/UiPlugins/service.ts';
import { hrefFor, sectionPath, parseAdminPath } from '../screens/Admin/shell/router.ts';

function mark(): void {
  const meta = document.createElement('meta');
  meta.name = 'mw-console';
  meta.content = 'admin';
  document.head.append(meta);
}

function unmark(): void {
  document.head.querySelectorAll('meta[name="mw-console"]').forEach((m) => m.remove());
}

function setBase(base: string | undefined): void {
  const g = globalThis as { __MW_BASE__?: string };
  if (base === undefined) delete g.__MW_BASE__;
  else g.__MW_BASE__ = base;
}

type Call = [string, RequestInit | undefined];

/** Stub `fetch`, run `fn`, and return the URLs it asked for. */
async function urlsOf(fn: () => Promise<unknown>, status = 200, body: unknown = {}): Promise<string[]> {
  const calls: Call[] = [];
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: string, init?: RequestInit) => {
      calls.push([url, init]);
      return new Response(status === 204 ? null : JSON.stringify(body), {
        status,
        headers: { 'content-type': 'application/json' },
      });
    }),
  );
  await fn().catch(() => undefined);
  return calls.map(([u]) => u);
}

beforeEach(() => {
  unmark();
  setBase(undefined);
  history.replaceState(null, '', '/');
});

afterEach(() => {
  unmark();
  setBase(undefined);
  history.replaceState(null, '', '/');
  vi.unstubAllGlobals();
});

describe('consolePath()', () => {
  it('is /admin on a page the server did not mark', () => {
    history.replaceState(null, '', '/ops-7f3k');
    expect(consolePath()).toBe('/admin');
    expect(isConsolePage()).toBe(false);
  });

  it('is the page address on the marked console page', () => {
    history.replaceState(null, '', '/ops-7f3k#/users');
    mark();
    expect(consolePath()).toBe('/ops-7f3k');
    expect(isConsolePage()).toBe(true);
  });

  it('drops the deploy prefix and a trailing slash, and keeps every segment', () => {
    setBase('/mail/');
    history.replaceState(null, '', '/mail/ops/console/');
    mark();
    expect(consolePath()).toBe('/ops/console');
    expect(adminUrl('/users')).toBe('/mail/ops/console/users');
  });

  it('the unmarked default page still counts as the console (dev server, older server)', () => {
    history.replaceState(null, '', '/admin');
    expect(isConsolePage()).toBe(true);
    history.replaceState(null, '', '/');
    expect(isConsolePage()).toBe(false);
  });

  it('recognises console locations', () => {
    history.replaceState(null, '', '/ops-7f3k');
    mark();
    expect(isConsoleLocation('/ops-7f3k')).toBe(true);
    expect(isConsoleLocation('/ops-7f3k#/users')).toBe(true);
    expect(isConsoleLocation('/admin#/users')).toBe(false);
    expect(isConsoleLocation('/ops-7f3kx')).toBe(false);
  });
});

describe('every admin client follows the console page', () => {
  beforeEach(() => {
    setBase('/mail');
    history.replaceState(null, '', '/mail/ops-7f3k');
    mark();
  });

  const at = (p: string): string => `/mail/ops-7f3k${p}`;

  it('admin slice', async () => {
    expect(await urlsOf(() => createHttpAdminApi().listDomains(), 200, [])).toEqual([at('/domains')]);
    expect(await urlsOf(() => createHttpAdminApi().session(), 401)).toEqual([at('/session')]);
  });

  it('plugins', async () => {
    expect(await urlsOf(() => createHttpPluginsApi().list(), 200, [])).toEqual([at('/plugins')]);
  });

  it('maintenance and search index', async () => {
    expect(await urlsOf(() => createHttpMaintenanceApi().rethread('a1'))).toEqual([at('/maintenance/rethread')]);
    expect(await urlsOf(() => createHttpSearchIndexApi().status())).toEqual([at('/maintenance/search-index')]);
  });

  it('SSO admin', async () => {
    expect(await urlsOf(() => createHttpSsoAdminApi().list(), 200, [])).toEqual([at('/sso')]);
  });

  it('two-factor policy', async () => {
    expect(await urlsOf(() => createHttpTwofaPolicyApi().list(), 200, [])).toEqual([at('/2fa/policy')]);
  });

  it('sign-in second step, who am I, search and dashboard', async () => {
    expect(await urlsOf(() => createAdminTotpVerifier()('pt', '123456'), 200, { username: 'x' })).toEqual([
      at('/login/totp'),
    ]);
    expect(await urlsOf(() => createAdminMeLoader()(), 404)).toEqual([at('/me')]);
    expect(await urlsOf(() => createAdminSearch()('q', new AbortController().signal), 200, { hits: [] })).toEqual([
      at('/search?q=q'),
    ]);
    expect(await urlsOf(() => createDashboardLoader()(), 404)).toEqual([at('/dashboard')]);
  });

  it('Assist and UI plugins', async () => {
    expect(await urlsOf(() => new AdminAssistApi().status())).toEqual([at('/assist/status')]);
    expect(await urlsOf(() => createHttpUiPluginsApi().list(), 200, [])).toEqual([at('/ui-plugins')]);
  });

  it('console links and sections', () => {
    expect(sectionPath('dashboard')).toBe('/ops-7f3k');
    expect(sectionPath('users')).toBe('/ops-7f3k#/users');
    expect(hrefFor(sectionPath('users'))).toBe('/mail/ops-7f3k#/users');
    expect(parseAdminPath('/ops-7f3k#/domains/x')).toEqual({ section: 'domains', rest: ['x'] });
  });
});
