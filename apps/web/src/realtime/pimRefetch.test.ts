import { describe, it, expect } from 'vitest';
import { pimTypesInChange, shouldRefetchPim } from './pimRefetch.ts';
import type { StateChange } from '../contracts/push.ts';

const coarsePing: StateChange = {
  '@type': 'StateChange',
  // The ping t5-e8 broadcasts on every PIM mutation: only mail keys move (or
  // none), no granular PIM keys — its ARRIVAL is the signal.
  changed: { acct1: { Email: 'e2', Mailbox: 'm2' } },
};

const granularCalendar: StateChange = {
  '@type': 'StateChange',
  changed: { acct1: { CalendarEvent: 'ce9' } },
};

describe('pimRefetch', () => {
  it('reports PIM types named in a change (none for the coarse ping)', () => {
    expect(pimTypesInChange(coarsePing)).toEqual([]);
    expect(pimTypesInChange(granularCalendar)).toEqual(['CalendarEvent']);
  });

  it('refetches the open PIM module on the coarse ping', () => {
    expect(shouldRefetchPim('calendar', coarsePing)).toBe(true);
    expect(shouldRefetchPim('tasks', coarsePing)).toBe(true);
    expect(shouldRefetchPim('notes', coarsePing)).toBe(true);
    expect(shouldRefetchPim('contacts', coarsePing)).toBe(true);
  });

  it('never refetches for a non-PIM surface', () => {
    expect(shouldRefetchPim('mail', coarsePing)).toBe(false);
    expect(shouldRefetchPim('outbox', granularCalendar)).toBe(false);
    expect(shouldRefetchPim('attachments', coarsePing)).toBe(false);
  });

  it('honors granular PIM keys precisely when present', () => {
    // A CalendarEvent change refetches calendar, not notes/contacts.
    expect(shouldRefetchPim('calendar', granularCalendar)).toBe(true);
    expect(shouldRefetchPim('notes', granularCalendar)).toBe(false);
    expect(shouldRefetchPim('contacts', granularCalendar)).toBe(false);
    // A Calendar change is relevant to tasks (VTODO lists are Calendar rows).
    const calChange: StateChange = { '@type': 'StateChange', changed: { acct1: { Calendar: 'c2' } } };
    expect(shouldRefetchPim('tasks', calChange)).toBe(true);
  });
});
