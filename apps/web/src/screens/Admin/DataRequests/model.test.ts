import { describe, expect, it } from 'vitest';
import { actionsFor, daysLeft, logFileName, onTimePercent } from './model.ts';

const NOW = 1_000_000;
const DAY = 86_400;
const none = { active: false } as const;

describe('daysLeft', () => {
  it('counts whole days up to the due date and past it', () => {
    expect(daysLeft({ dueAt: NOW + 3 * DAY - 10, dueState: 'ok' }, NOW)).toBe(3);
    expect(daysLeft({ dueAt: NOW, dueState: 'dueSoon' }, NOW)).toBe(0);
    expect(daysLeft({ dueAt: NOW - 2 * DAY + 5, dueState: 'overdue' }, NOW)).toBe(-2);
  });
  it('is null once the request is answered', () => {
    expect(daysLeft({ dueAt: NOW + DAY, dueState: 'none' }, NOW)).toBeNull();
  });
});

describe('actionsFor', () => {
  it('offers the identity check while waiting for it, and approval while waiting for approval', () => {
    expect(actionsFor({ status: 'awaiting_verification', kind: 'access', legalHold: none })).toMatchObject({ verify: true, approve: false, reject: true });
    expect(actionsFor({ status: 'awaiting_approval', kind: 'access', legalHold: none })).toMatchObject({ verify: false, approve: true, extend: true });
  });
  it('does not offer approval of a blocked erasure while a legal hold applies', () => {
    const held = { active: true, source: 'policy' } as const;
    expect(actionsFor({ status: 'blocked', kind: 'erasure', legalHold: held })).toMatchObject({ approve: false, release: false, hold: false });
    expect(actionsFor({ status: 'blocked', kind: 'erasure', legalHold: { active: true, source: 'request' } })).toMatchObject({ release: true });
    expect(actionsFor({ status: 'blocked', kind: 'erasure', legalHold: none })).toMatchObject({ approve: true });
  });
  it('lets an administrator complete only a request in progress and hold only an open erasure', () => {
    expect(actionsFor({ status: 'in_progress', kind: 'objection', legalHold: none })).toMatchObject({ complete: true, hold: false });
    expect(actionsFor({ status: 'scheduled', kind: 'erasure', legalHold: none })).toMatchObject({ complete: false, hold: true, run: true });
  });
  it('offers nothing on a finished request', () => {
    const a = actionsFor({ status: 'completed', kind: 'erasure', legalHold: none });
    expect(Object.values(a).some(Boolean)).toBe(false);
  });
});

describe('onTimePercent and logFileName', () => {
  it('rounds the share and is null with nothing answered', () => {
    expect(onTimePercent({ answered: 3, onTime: 2 })).toBe(67);
    expect(onTimePercent({ answered: 0, onTime: 0 })).toBeNull();
  });
  it('names the log by date', () => {
    expect(logFileName(new Date('2026-11-05T10:00:00Z'))).toBe('data-requests-2026-11-05.csv');
  });
});
