import { describe, it, expect } from 'vitest';
import { parseHash, routeHash, isPimSurface, PIM_SURFACES } from './router.ts';

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
