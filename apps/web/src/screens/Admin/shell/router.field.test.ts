// `?field=` deep links and fragment queries (t32-e71a, UX10).
import { describe, it, expect, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createAdminRouter, fieldPath, parseAdminPath, sectionPath } from './router.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

describe('fragment queries', () => {
  it('parses ?field= into the route and keeps the other parameters as the query', () => {
    expect(parseAdminPath('/admin#/config?field=smtp.port')).toEqual({
      section: 'config',
      rest: [],
      query: { field: 'smtp.port' },
      field: 'smtp.port',
    });
    expect(parseAdminPath('/admin#/audit?target=setting%3Asignin.newUsers')).toEqual({
      section: 'audit',
      rest: [],
      query: { target: 'setting:signin.newUsers' },
    });
    expect(parseAdminPath('/admin#/domains/example.org?field=mode').rest).toEqual(['example.org']);
  });

  it('builds the same locations it parses', () => {
    const p = fieldPath('config', 'upstream.proxyTimeoutSecs');
    expect(p).toBe('/admin#/config?field=upstream.proxyTimeoutSecs');
    expect(parseAdminPath(p).field).toBe('upstream.proxyTimeoutSecs');
    expect(sectionPath('audit', [], { target: 'setting:x y' })).toBe('/admin#/audit?target=setting%3Ax+y');
    expect(fieldPath('dashboard', 'a')).toBe('/admin#/dashboard?field=a');
    expect(sectionPath('users')).toBe('/admin#/users');
  });

  it('a location without a query parses as before', () => {
    expect(parseAdminPath('/admin#/users')).toEqual({ section: 'users', rest: [] });
  });
});

describe('navigation that changes only the query', () => {
  it('a field link on the open page does not ask the leave question, and updates the route', async () => {
    await createRoot(async (dispose) => {
      let asked = 0;
      const r = createAdminRouter(async () => {
        asked += 1;
        return true;
      });
      await r.navigate('/admin#/config');
      expect(asked).toBe(1);
      expect(await r.navigate(fieldPath('config', 'smtp.port'))).toBe(true);
      expect(asked).toBe(1);
      expect(r.route().field).toBe('smtp.port');
      expect(location.hash).toBe('#/config?field=smtp.port');
      r.dispose();
      dispose();
    });
  });

  it('a field link to another section asks, and a refusal keeps the page', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter(async () => false);
      expect(await r.navigate(fieldPath('users', 'quota'))).toBe(false);
      expect(r.route()).toEqual({ section: 'dashboard', rest: [] });
      r.dispose();
      dispose();
    });
  });

  it('the same field link twice is a new route each time it changes', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter();
      await r.navigate(fieldPath('config', 'a'));
      expect(r.route().field).toBe('a');
      await r.navigate(fieldPath('config', 'b'));
      expect(r.route().field).toBe('b');
      r.dispose();
      dispose();
    });
  });

  it('a query typed into the address bar on the open page updates the route without asking', async () => {
    await createRoot(async (dispose) => {
      let asked = 0;
      const r = createAdminRouter(async () => {
        asked += 1;
        return true;
      });
      await r.navigate('/admin#/config');
      history.replaceState(null, '', '/admin#/config?field=x');
      window.dispatchEvent(new HashChangeEvent('hashchange'));
      await Promise.resolve();
      expect(r.route().field).toBe('x');
      expect(asked).toBe(1);
      r.dispose();
      dispose();
    });
  });
});
