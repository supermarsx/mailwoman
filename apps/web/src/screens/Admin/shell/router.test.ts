import { describe, it, expect, beforeEach } from 'vitest';
import { createRoot } from 'solid-js';
import { createAdminRouter, parseAdminPath, sectionPath } from './router.ts';

beforeEach(() => history.replaceState(null, '', '/admin'));

describe('admin locations', () => {
  it('the dashboard is plain /admin; sections live in the fragment', () => {
    expect(parseAdminPath('/admin')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin#')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin#/')).toEqual({ section: 'dashboard', rest: [] });
    expect(parseAdminPath('/admin#/users')).toEqual({ section: 'users', rest: [] });
    expect(parseAdminPath('/admin#/domains/b%C3%BCcher.example/')).toEqual({ section: 'domains', rest: ['bücher.example'] });
  });

  it('never puts a section in the path (the JSON API owns /admin/<x>, and chunks load relative to the page)', () => {
    expect(parseAdminPath('/admin/users')).toEqual({ section: 'dashboard', rest: [] });
    for (const id of ['users', 'domains', 'plugins']) expect(sectionPath(id).split('#')[0]).toBe('/admin');
  });

  it('builds and parses the same location', () => {
    expect(sectionPath('dashboard')).toBe('/admin');
    const p = sectionPath('domains', ['a b/c']);
    expect(p).toBe('/admin#/domains/a%20b%2Fc');
    expect(parseAdminPath(p)).toEqual({ section: 'domains', rest: ['a b/c'] });
  });
});

describe('createAdminRouter', () => {
  it('navigate pushes a history entry and updates the route', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter();
      const before = history.length;
      expect(await r.navigate('/admin#/users')).toBe(true);
      expect(location.pathname + location.hash).toBe('/admin#/users');
      expect(history.length).toBe(before + 1);
      expect(r.route().section).toBe('users');
      r.dispose();
      dispose();
    });
  });

  it('a refusing leave check keeps the page and the URL', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter(async () => false);
      expect(await r.navigate('/admin#/users')).toBe(false);
      expect(location.pathname + location.hash).toBe('/admin');
      expect(r.route().section).toBe('dashboard');
      r.dispose();
      dispose();
    });
  });

  it('Back with a refusing leave check puts the shown page back in the address bar', async () => {
    await createRoot(async (dispose) => {
      let allow = true;
      const r = createAdminRouter(async () => allow);
      await r.navigate('/admin#/users');
      allow = false;
      history.replaceState(null, '', '/admin');
      window.dispatchEvent(new PopStateEvent('popstate'));
      await Promise.resolve();
      await Promise.resolve();
      expect(location.pathname + location.hash).toBe('/admin#/users');
      expect(r.route().section).toBe('users');
      r.dispose();
      dispose();
    });
  });

  it('a fragment typed into the address bar (hashchange) opens that section', async () => {
    await createRoot(async (dispose) => {
      const r = createAdminRouter();
      history.replaceState(null, '', '/admin#/domains');
      window.dispatchEvent(new HashChangeEvent('hashchange'));
      await Promise.resolve();
      await Promise.resolve();
      expect(r.route().section).toBe('domains');
      r.dispose();
      dispose();
    });
  });
});
