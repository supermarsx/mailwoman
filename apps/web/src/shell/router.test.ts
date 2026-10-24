import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import {
  createShellRouter,
  isPimSurface,
  isUnknownRoute,
  parseHash,
  PIM_SURFACES,
  routeHash,
  type ShellRouter,
} from './router.ts';

describe('shell router', () => {
  it('parses the mail-family + PIM routes', () => {
    expect(parseHash('')).toEqual({ surface: 'mail', param: null });
    expect(parseHash('#/mail')).toEqual({ surface: 'mail', param: null });
    expect(parseHash('#/tasks')).toEqual({ surface: 'tasks', param: null });
    expect(parseHash('#/calendar/week')).toEqual({ surface: 'calendar', param: 'week' });
    expect(parseHash('#/notes/abc123')).toEqual({ surface: 'notes', param: 'abc123' });
    expect(parseHash('#/contacts/c-9')).toEqual({ surface: 'contacts', param: 'c-9' });
  });

  it('falls back to mail for unknown surfaces', () => {
    expect(parseHash('#/nope')).toEqual({ surface: 'mail', param: null });
    expect(parseHash('#/settings/x')).toEqual({ surface: 'mail', param: null });
  });

  it('round-trips a route through routeHash → parseHash', () => {
    for (const s of ['calendar', 'tasks', 'notes', 'contacts'] as const) {
      expect(parseHash(routeHash(s))).toEqual({ surface: s, param: null });
      expect(parseHash(routeHash(s, 'p'))).toEqual({ surface: s, param: 'p' });
    }
  });

  it('encodes route params (id with special chars) and round-trips them', () => {
    // encodeURIComponent encodes the slash, so the id survives intact.
    const h = routeHash('notes', 'a/b c');
    expect(parseHash(h)).toEqual({ surface: 'notes', param: 'a/b c' });
    expect(routeHash('notes', 'a b')).toBe('#/notes/a%20b');
    expect(parseHash('#/notes/a%20b')).toEqual({ surface: 'notes', param: 'a b' });
  });

  it('classifies PIM vs mail surfaces', () => {
    expect(PIM_SURFACES).toEqual(['calendar', 'tasks', 'notes', 'contacts']);
    expect(isPimSurface('calendar')).toBe(true);
    expect(isPimSurface('contacts')).toBe(true);
    expect(isPimSurface('mail')).toBe(false);
    expect(isPimSurface('outbox')).toBe(false);
  });
});

// t32-h10: an address that names no surface lands on the inbox with a notice,
// and a malformed one never throws (a throw here was a blank screen).
describe('shell router: unknown routes', () => {
  function withRouter(hash: string, run: (r: ShellRouter, notices: string[]) => void): void {
    history.replaceState(null, '', `/${hash}`);
    const notices: string[] = [];
    createRoot((dispose) => {
      const r = createShellRouter(() => notices.push(location.hash));
      run(r, notices);
      dispose();
    });
    history.replaceState(null, '', '/');
  }

  it('an unknown surface goes to the inbox, replacing the entry, and says so once', () => {
    const before = history.length;
    withRouter('#/bogus', (r, notices) => {
      expect(r.route()).toEqual({ surface: 'mail', param: null });
      expect(location.hash).toBe('#/mail');
      expect(history.length).toBe(before);
      expect(notices).toEqual(['#/mail']);
    });
  });

  it('an unknown surface typed later is handled the same way', () => {
    withRouter('#/tasks', (r, notices) => {
      expect(r.route().surface).toBe('tasks');
      history.replaceState(null, '', '/#/nope/x');
      window.dispatchEvent(new HashChangeEvent('hashchange'));
      expect(r.route()).toEqual({ surface: 'mail', param: null });
      expect(location.hash).toBe('#/mail');
      expect(notices).toHaveLength(1);
    });
  });

  it('known routes, the empty hash and non-route fragments are left alone', () => {
    for (const hash of ['', '#', '#/', '#/mail', '#/calendar/week', '#/keys', '#main']) {
      withRouter(hash, (_r, notices) => {
        expect(notices).toEqual([]);
      });
    }
  });

  it('a malformed escape in the parameter does not throw', () => {
    expect(() => parseHash('#/notes/%E0%A4%A')).not.toThrow();
    expect(parseHash('#/notes/%E0%A4%A')).toEqual({ surface: 'notes', param: null });
  });

  it('isUnknownRoute names only route-shaped fragments with an unknown head', () => {
    expect(isUnknownRoute('#/bogus')).toBe(true);
    expect(isUnknownRoute('#/settings/x')).toBe(true);
    expect(isUnknownRoute('#/mail')).toBe(false);
    expect(isUnknownRoute('#/')).toBe(false);
    expect(isUnknownRoute('')).toBe(false);
    expect(isUnknownRoute('#main')).toBe(false);
  });
});
