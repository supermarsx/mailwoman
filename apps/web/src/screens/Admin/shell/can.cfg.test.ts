// The t32-cfg §4.20 capabilities are in the console's catalogue (t32-e71a).
import { describe, it, expect } from 'vitest';
import { CAPABILITIES } from './can.ts';

const CFG = [
  'config.read', 'config.export', 'config.import', 'integrations.write', 'cache.purge', 'server_text.write',
  'maintenance.read', 'maintenance.write', 'maintenance.run', 'queue.manage', 'jobs.read', 'observability.read',
  'sessions.read', 'sessions.revoke', 'bans.read', 'bans.write', 'updates.read', 'updates.check', 'updates.apply',
  'backups.read', 'backups.run', 'about.manage', 'diagnostics.read', 'diagnostics.run', 'diagnostics.schedule',
  'diagnostics.mailpath', 'diagnostics.simulate', 'diagnostics.jmap_session', 'data_export.run',
];

describe('capability catalogue', () => {
  it('names every t32-cfg §4.20 capability once', () => {
    const all: readonly string[] = CAPABILITIES;
    expect(CFG.filter((c) => !all.includes(c))).toEqual([]);
    expect(new Set(all).size).toBe(all.length);
  });
});
