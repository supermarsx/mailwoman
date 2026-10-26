import { describe, expect, it } from 'vitest';
import { shouldNotify } from './notificationPolicy.ts';
import type { NotificationConfig } from '../screens/Settings/types.ts';
const config: NotificationConfig = { enabled: true, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00' } };
describe('notification delivery policy', () => {
  it('does not grant delivery without loaded enabled policy', () => {
    expect(shouldNotify(undefined, {})).toBe(false);
    expect(shouldNotify({ ...config, enabled: false }, {})).toBe(false);
    expect(shouldNotify(config, {})).toBe(true);
  });
  it('matches ordered sender, subject and mailbox rules without interpreting regex', () => {
    const rules = [ { id: '1', label: '', match: 'private', action: 'mute' as const }, { id: '2', label: '', match: 'boss@example.org', action: 'notify' as const } ];
    expect(shouldNotify({ ...config, rules }, { subject: 'PRIVATE report', from: [{ email: 'boss@example.org' }] })).toBe(false);
    expect(shouldNotify({ ...config, rules }, { from: [{ email: 'BOSS@example.org' }] })).toBe(true);
    expect(shouldNotify({ ...config, rules }, { mailboxIds: { private: true } })).toBe(false);
    expect(shouldNotify({ ...config, rules }, { mailboxNames: ['Private projects'] })).toBe(false);
  });
  it('uses the saved timezone including DST and overnight boundaries', () => {
    const local = { ...config, quietHours: { ...config.quietHours, enabled: true, timezone: 'Europe/London' } };
    expect(shouldNotify(local, {}, new Date('2026-07-01T21:00:00Z'))).toBe(false);
    expect(shouldNotify(local, {}, new Date('2026-07-02T06:00:00Z'))).toBe(true);
    expect(shouldNotify(local, {}, new Date('2026-01-01T21:00:00Z'))).toBe(true);
  });
  it('fails closed for malformed enabled quiet hours', () => {
    expect(shouldNotify({ ...config, quietHours: { enabled: true, start: 'bad', end: '07:00' } }, {})).toBe(false);
    expect(shouldNotify({ ...config, quietHours: { ...config.quietHours, enabled: true, timezone: 'invalid' } }, {})).toBe(false);
  });
});
