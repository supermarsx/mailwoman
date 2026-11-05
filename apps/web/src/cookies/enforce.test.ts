import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { installGate, purge, scanStorage, unclassified } from './enforce.ts';
import { resolve, type ConsentConfig } from './policy.ts';

const config: ConsentConfig = {
  enabled: true, mode: 'optIn', version: 1, lifetimeDays: 180, reprompt: true, unclassified: 'report',
  categories: [
    { id: 'necessary', enabled: true, default: true, label: {}, description: {} },
    { id: 'functional', enabled: true, default: false, label: {}, description: {} },
    { id: 'analytics', enabled: true, default: false, label: {}, description: {} },
  ],
  inventory: [
    { id: 'a', kind: 'localStorage', pattern: 'mw.theme.*', category: 'functional', purpose: '', duration: '', party: 'first', provider: '' },
    { id: 'b', kind: 'localStorage', pattern: 'mw.consent*', category: 'necessary', purpose: '', duration: '', party: 'first', provider: '' },
    { id: 'c', kind: 'cookie', pattern: 'mw_session', category: 'necessary', purpose: '', duration: '', party: 'first', provider: '' },
    { id: 'd', kind: 'cookie', pattern: 'pref_*', category: 'functional', purpose: '', duration: '', party: 'first', provider: '' },
  ],
  rules: [{ kind: 'localStorage', pattern: '_ga*', category: 'analytics' }],
};

let undo: (() => void) | null = null;
beforeEach(() => { localStorage.clear(); sessionStorage.clear(); });
afterEach(() => { undo?.(); undo = null; for (const name of ['pref_x', 'mw_session', 'seen']) document.cookie = `${name}=; Max-Age=0; Path=/`; });

describe('scanning', () => {
  it('lists names of cookies and storage, never values', async () => {
    localStorage.setItem('mw.theme.prefs', 'SECRET-VALUE');
    sessionStorage.setItem('tab', 'x');
    document.cookie = 'seen=SECRET-COOKIE; Path=/';
    const found = await scanStorage();
    expect(found).toContainEqual({ kind: 'localStorage', name: 'mw.theme.prefs' });
    expect(found).toContainEqual({ kind: 'sessionStorage', name: 'tab' });
    expect(found).toContainEqual({ kind: 'cookie', name: 'seen' });
    expect(JSON.stringify(found)).not.toContain('SECRET');
  });

  it('reports only names that no inventory entry or rule classifies, once', () => {
    const items = [
      { kind: 'localStorage' as const, name: 'mw.theme.prefs' },
      { kind: 'localStorage' as const, name: '_ga_1' },
      { kind: 'localStorage' as const, name: 'vendor.key' },
      { kind: 'localStorage' as const, name: 'vendor.key' },
      { kind: 'cookie' as const, name: 'vendor.key' },
    ];
    expect(unclassified(items, config)).toEqual([{ kind: 'localStorage', name: 'vendor.key' }, { kind: 'cookie', name: 'vendor.key' }]);
  });
});

describe('enforcement', () => {
  it('blocks writes of storage that is not allowed, and lets necessary and unknown names through', () => {
    const effective = resolve(config, null, { gpc: false, dnt: false }, Date.now());
    undo = installGate(() => ({ config, effective }));
    localStorage.setItem('mw.theme.prefs', 'dark');
    localStorage.setItem('_ga_1', 'x');
    localStorage.setItem('mw.consent', '{}');
    localStorage.setItem('vendor.key', 'v');
    sessionStorage.setItem('mw.theme.tab', 'x');
    expect(localStorage.getItem('mw.theme.prefs')).toBeNull();
    expect(localStorage.getItem('_ga_1')).toBeNull();
    expect(localStorage.getItem('mw.consent')).toBe('{}');
    expect(localStorage.getItem('vendor.key')).toBe('v');
    // The inventory entry is for localStorage; the same name in sessionStorage is a different, unclassified item.
    expect(sessionStorage.getItem('mw.theme.tab')).toBe('x');
  });

  it('lets a category through once the visitor allows it', () => {
    let accepted = false;
    undo = installGate(() => ({
      config,
      effective: resolve(config, accepted ? { version: 1, decision: 'custom', categories: { necessary: true, functional: true, analytics: false, marketing: false }, signal: 'none', decidedAt: Date.now() } : null, { gpc: false, dnt: false }, Date.now()),
    }));
    localStorage.setItem('mw.theme.prefs', 'a');
    expect(localStorage.getItem('mw.theme.prefs')).toBeNull();
    accepted = true;
    localStorage.setItem('mw.theme.prefs', 'b');
    expect(localStorage.getItem('mw.theme.prefs')).toBe('b');
  });

  it('blocks cookie writes of a category that is not allowed but never blocks removal', () => {
    const effective = resolve(config, null, { gpc: false, dnt: false }, Date.now());
    undo = installGate(() => ({ config, effective }));
    document.cookie = 'pref_x=1; Path=/';
    document.cookie = 'mw_session=1; Path=/';
    expect(document.cookie).not.toContain('pref_x');
    expect(document.cookie).toContain('mw_session=1');
    document.cookie = 'mw_session=; Max-Age=0; Path=/';
    expect(document.cookie).not.toContain('mw_session');
  });

  it('purges what is already stored in categories that are not allowed', async () => {
    localStorage.setItem('mw.theme.prefs', 'dark');
    localStorage.setItem('_ga_1', 'x');
    localStorage.setItem('mw.consent', '{}');
    localStorage.setItem('vendor.key', 'v');
    document.cookie = 'pref_x=1; Path=/';
    const effective = resolve(config, null, { gpc: false, dnt: false }, Date.now());
    const removed = await purge(config, effective);
    expect(removed.map((r) => r.name).sort()).toEqual(['_ga_1', 'mw.theme.prefs', 'pref_x']);
    expect(localStorage.getItem('mw.theme.prefs')).toBeNull();
    expect(localStorage.getItem('mw.consent')).toBe('{}');
    expect(localStorage.getItem('vendor.key')).toBe('v');
    expect(document.cookie).not.toContain('pref_x');
  });

  it('does nothing while the control is off', async () => {
    localStorage.setItem('mw.theme.prefs', 'dark');
    expect(await purge({ enabled: false }, resolve({ enabled: false }, null, { gpc: false, dnt: false }, Date.now()))).toEqual([]);
    expect(localStorage.getItem('mw.theme.prefs')).toBe('dark');
  });
});
