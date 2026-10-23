import { describe, it, expect, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { CONSOLE_PREFIX, createAdminRouter, parseAdminPath, sectionPath } from './router.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

describe('admin paths', () => {
  it('the dashboard is /admin; sections live under /admin/console', () => {
    expect(parseAdminPath('/admin')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin/')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin/console')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin/console/users')).toEqual({ section: 'users', rest: [] });
    expect(parseAdminPath('/admin/console/domains/b%C3%BCcher.example/')).toEqual({
      section: 'domains',
      rest: ['bücher.example'],
    });
  });

  it('never routes into the API namespace (/admin/users is the JSON API, not a section)', () => {
    expect(parseAdminPath('/admin/users')).toEqual({ section: 'dashboard', rest: [] });
    expect(sectionPath('users').startsWith(`${CONSOLE_PREFIX}/`)).toBe(true);
  });

  it('builds and parses the same path', () => {
    expect(sectionPath('dashboard')).toBe('/admin');
    const p = sectionPath('domains', ['a b/c']);
    expect(p).toBe('/admin/console/domains/a%20b%2Fc');
    expect(parseAdminPath(p)).toEqual({ section: 'domains', rest: ['a b/c'] });
  });
});

describe('createAdminRouter', () => {
  it('navigate pushes a history entry and updates the route', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter();
      const before = history.length;
      expect(await r.navigate('/admin/console/users')).toBe(true);
      expect(location.pathname).toBe('/admin/console/users');
      expect(history.length).toBe(before + 1);
      expect(r.route().section).toBe('users');
      r.dispose();
      dispose();
    });
  });

  it('a refusing leave check keeps the page and the URL', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter(async () => false);
      expect(await r.navigate('/admin/console/users')).toBe(false);
      expect(location.pathname).toBe('/admin');
      expect(r.route().section).toBe('dashboard');
      r.dispose();
      dispose();
    });
  });

  it('Back with a refusing leave check puts the shown page back in the address bar', async () => {
    await createRoot(async (dispose) => {
      let allow = true;
      const r = createAdminRouter(async () => allow);
      await r.navigate('/admin/console/users');
      allow = false;
      history.replaceState(null, '', '/admin');
      window.dispatchEvent(new PopStateEvent('popstate'));
      await Promise.resolve();
      await Promise.resolve();
      expect(location.pathname).toBe('/admin/console/users');
      expect(r.route().section).toBe('users');
      r.dispose();
      dispose();
    });
  });
});
