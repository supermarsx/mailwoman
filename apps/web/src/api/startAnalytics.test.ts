import { describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createShellRouter } from '../shell/router.ts';
import { ANALYTICS_CHANNEL, announceAnalyticsRevoke, createTagGuard, documentAnalyticsTag, observeShellRoute, severThirdPartyNetwork, startAnalytics } from './startAnalytics.ts';

// t33-e10 (t32-h10): analytics started at app boot used a second shell router,
// which replaced an unknown route with the inbox before the shell's router saw
// it, so the shell's "That page does not exist" notice never showed.
describe('startAnalytics', () => {
  it('leaves an unknown route for the shell router, which then reports it', () => {
    history.replaceState(null, '', '/#/bogus');
    let unknown = 0;
    createRoot((dispose) => {
      startAnalytics();
      expect(location.hash).toBe('#/bogus');
      createShellRouter(() => { unknown += 1; });
      dispose();
    });
    expect(unknown).toBe(1);
    expect(location.hash).toBe('#/mail');
  });

  it('follows the route without writing it', () => {
    history.replaceState(null, '', '/#/mail');
    createRoot((dispose) => {
      const route = observeShellRoute();
      expect(route().surface).toBe('mail');
      location.hash = '#/calendar';
      window.dispatchEvent(new HashChangeEvent('hashchange'));
      expect(route().surface).toBe('calendar');
      expect(location.hash).toBe('#/calendar');
      dispose();
    });
  });
});

// t63: a loaded provider tag cannot be unloaded, so a withdrawal cuts the page
// off from third-party network at once and reloads it once the change is saved.
const KEY = `plausible:${'a'.repeat(64)}`;
describe('browser analytics tag guard', () => {
  it('reads the consent key from the server marker', () => {
    const d = document.implementation.createHTMLDocument('t');
    expect(documentAnalyticsTag(d)).toBeNull();
    d.head.innerHTML = `<meta name="mw-analytics" content="https://plausible.io" data-consent="${KEY}">`;
    expect(documentAnalyticsTag(d)).toBe(KEY);
    d.head.innerHTML = '<meta name="mw-analytics" content="https://plausible.io" data-consent="true">';
    expect(documentAnalyticsTag(d)).toBeNull();
  });
  it('withdrawal severs at once and reloads as soon as it is saved, without a navigation', () => {
    const reload = vi.fn(); const sever = vi.fn();
    const guard = createTagGuard(KEY, reload, sever);
    guard.account({ id: 'a', generation: 1, storageAuthority: 'auth-aaaaaaaaaaaaaaaa' });
    guard.consent(undefined, false, 'saving');
    guard.consent(KEY, false, 'synced');
    expect(sever).not.toHaveBeenCalled();
    guard.consent('', false, 'saving');
    expect(sever).toHaveBeenCalledTimes(1);
    expect(reload).not.toHaveBeenCalled();
    guard.consent('', false, 'synced');
    expect(reload).toHaveBeenCalledTimes(1);
    guard.consent('', false, 'synced');
    expect(reload).toHaveBeenCalledTimes(1);
  });
  it('treats counts-only or another provider\'s consent as no consent', () => {
    const reload = vi.fn(); const sever = vi.fn();
    const guard = createTagGuard(KEY, reload, sever);
    guard.consent(`custom:${'b'.repeat(64)}`, false, 'synced');
    expect(sever).toHaveBeenCalled(); expect(reload).toHaveBeenCalled();
  });
  it('a privacy signal severs and reloads immediately', () => {
    const reload = vi.fn(); const sever = vi.fn();
    createTagGuard(KEY, reload, sever).consent(KEY, true, 'saving');
    expect(sever).toHaveBeenCalled(); expect(reload).toHaveBeenCalled();
  });
  it('reloads at once when another account appears in the tab', () => {
    const reload = vi.fn();
    const guard = createTagGuard(KEY, reload);
    guard.account({ id: 'a', generation: 1 });
    guard.account(null);
    guard.account({ id: 'a', generation: 2 });
    expect(reload).not.toHaveBeenCalled();
    guard.account({ id: 'b', generation: 3 });
    expect(reload).toHaveBeenCalledTimes(1);
  });
  it('does nothing in a document served without a tag', () => {
    const reload = vi.fn(); const sever = vi.fn();
    const guard = createTagGuard(null, reload, sever);
    guard.account({ id: 'a', generation: 1 }); guard.account({ id: 'b', generation: 2 });
    guard.consent('', true, 'synced');
    expect(reload).not.toHaveBeenCalled(); expect(sever).not.toHaveBeenCalled();
  });
  it('severing blocks third-party beacons, fetch and XHR but keeps same-origin requests', async () => {
    const calls: string[] = [];
    const win = { location: new URL('https://mail.example.test/#/mail'), navigator: { sendBeacon: () => true },
      fetch: (input: string) => { calls.push(String(input)); return Promise.resolve(new Response('')); },
      XMLHttpRequest: class { open(_m: string, url: string) { calls.push(`xhr ${url}`); } } } as unknown as Window & typeof globalThis;
    severThirdPartyNetwork(win);
    expect(win.navigator.sendBeacon('https://plausible.io/api/event', '')).toBe(false);
    await expect(win.fetch('https://plausible.io/api/event')).rejects.toThrow('withdrawn');
    await win.fetch('/api/me');
    const xhr = new win.XMLHttpRequest();
    expect(() => xhr.open('POST', 'https://collect.example.com/x')).toThrow('withdrawn');
    xhr.open('GET', '/api/analytics/status');
    expect(calls).toEqual(['/api/me', 'xhr /api/analytics/status']);
  });
});

// t63 R4: withdrawal and sign-ins reach every tab of this browser.
describe('cross-tab revocation', () => {
  it('a withdrawal here tells the other tabs, once', () => {
    const announce = vi.fn();
    const guard = createTagGuard(KEY, vi.fn(), vi.fn(), announce);
    guard.consent('', false, 'saving');
    guard.consent('', false, 'synced');
    expect(announce).toHaveBeenCalledTimes(1);
  });
  it('a failed save of this tab\'s withdrawal stays cut off and retries with backoff, never reloading (F1)', () => {
    const reload = vi.fn(); const announce = vi.fn(); const sever = vi.fn(); const retry = vi.fn();
    const guard = createTagGuard(KEY, reload, sever, announce, retry);
    guard.consent(KEY, false, 'synced');
    for (let i = 0; i < 5; i += 1) { guard.consent('', false, 'saving'); guard.consent('', false, 'error'); }
    expect(reload).not.toHaveBeenCalled();
    expect(announce).not.toHaveBeenCalled();
    expect(sever).toHaveBeenCalledTimes(1);
    expect(retry.mock.calls.map(c => c[0])).toEqual([0, 1, 2, 3, 4]);
    // Once a retry is saved, it is a confirmed withdrawal.
    guard.consent('', false, 'synced');
    expect(reload).toHaveBeenCalledTimes(1);
    expect(announce).toHaveBeenCalledTimes(1);
  });
  it('a failed or empty preferences LOAD never cuts off, announces or reloads (no loop)', () => {
    const reload = vi.fn(); const sever = vi.fn(); const announce = vi.fn();
    const guard = createTagGuard(KEY, reload, sever, announce);
    for (let i = 0; i < 50; i += 1) { guard.consent('', false, 'error'); guard.consent('', false, 'saving'); }
    expect(reload).not.toHaveBeenCalled();
    expect(sever).not.toHaveBeenCalled();
    expect(announce).not.toHaveBeenCalled();
  });
  it('a revoke from another tab cuts off and reloads a tagged tab', () => {
    const reload = vi.fn(); const sever = vi.fn();
    createTagGuard(KEY, reload, sever).revoked();
    expect(sever).toHaveBeenCalled(); expect(reload).toHaveBeenCalled();
    const untagged = vi.fn();
    createTagGuard(null, untagged).revoked();
    expect(untagged).not.toHaveBeenCalled();
  });
  it('another account signing in announces it, even from an untagged tab', () => {
    const announce = vi.fn();
    const guard = createTagGuard(null, vi.fn(), vi.fn(), announce);
    guard.account({ id: 'a', generation: 1 });
    expect(announce).not.toHaveBeenCalled();
    guard.account({ id: 'b', generation: 2 });
    expect(announce).toHaveBeenCalledTimes(1);
  });
  it('posts a revoke other tabs receive', async () => {
    const other = new BroadcastChannel(ANALYTICS_CHANNEL);
    const got = new Promise<unknown>(resolve => { other.onmessage = event => resolve(event.data); });
    announceAnalyticsRevoke();
    await expect(got).resolves.toEqual({ type: 'revoke' });
    other.close();
  });
});
