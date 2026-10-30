import { describe, it, expect, vi, beforeEach } from 'vitest';
import { prefetchAllowed, resetIdlePrefetch, scheduleIdlePrefetch, READER_CATALOGS, SETTINGS_CATALOGS } from './idlePrefetch.ts';

function host(extra: Record<string, unknown> = {}) {
  const idle: (() => void)[] = [];
  return {
    idle,
    host: { requestIdleCallback: (cb: () => void) => { idle.push(cb); return idle.length; }, setTimeout: vi.fn(), ...extra },
  };
}
const flush = () => new Promise((r) => setTimeout(r, 0));

describe('idlePrefetch (t99)', () => {
  beforeEach(() => resetIdlePrefetch());

  it('runs the steps one at a time, each in its own idle period, and only once per page', async () => {
    const { idle, host: h } = host();
    const order: number[] = [];
    const steps = [async () => { order.push(1); }, async () => { order.push(2); }];
    expect(scheduleIdlePrefetch(steps, h as never)).toBe(true);
    expect(order).toEqual([]);
    idle.shift()!();
    await flush();
    expect(order).toEqual([1]);
    idle.shift()!();
    await flush();
    expect(order).toEqual([1, 2]);
    expect(scheduleIdlePrefetch(steps, h as never)).toBe(false);
  });

  it('a failing step does not stop the next one', async () => {
    const { idle, host: h } = host();
    const ran: string[] = [];
    scheduleIdlePrefetch([async () => { throw new Error('chunk gone'); }, async () => { ran.push('next'); }], h as never);
    idle.shift()!();
    await flush();
    idle.shift()!();
    await flush();
    expect(ran).toEqual(['next']);
  });

  it('does nothing when the user asked to save data or the network is 2G', () => {
    expect(prefetchAllowed({ setTimeout, navigator: { connection: { saveData: true } } } as never)).toBe(false);
    expect(prefetchAllowed({ setTimeout, navigator: { connection: { effectiveType: '2g' } } } as never)).toBe(false);
    expect(prefetchAllowed({ setTimeout, navigator: { connection: { effectiveType: '4g' } } } as never)).toBe(true);
    const { host: h } = host({ navigator: { connection: { saveData: true } } });
    expect(scheduleIdlePrefetch([async () => undefined], h as never)).toBe(false);
  });

  it('names catalogs that exist', async () => {
    const files = import.meta.glob('../../locales/en/*.ftl', { query: '?raw', import: 'default' });
    for (const name of [...READER_CATALOGS, ...SETTINGS_CATALOGS]) expect(Object.keys(files)).toContain(`../../locales/en/${name}.ftl`);
  });
});
