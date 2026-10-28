import { describe, expect, it } from 'vitest';
import { createRoot } from 'solid-js';
import { createShellRouter } from '../shell/router.ts';
import { observeShellRoute, startAnalytics } from './startAnalytics.ts';

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
