import { describe, it, expect } from 'vitest';
import { createConnection } from './connection.ts';

describe('createConnection', () => {
  it('starts offline until the push client reports', () => {
    const c = createConnection();
    expect(c.state()).toBe('offline');
    expect(c.transport()).toBe('offline');
  });

  it('maps push lifecycle to connection state', () => {
    const c = createConnection();
    c.report('connecting', 'ws');
    expect(c.state()).toBe('connecting');
    c.report('open', 'ws');
    expect(c.state()).toBe('online');
    expect(c.transport()).toBe('ws');
    c.report('reconnecting', 'ws');
    expect(c.state()).toBe('connecting');
    c.report('degraded', 'poll');
    expect(c.state()).toBe('degraded');
    expect(c.transport()).toBe('poll');
    c.report('closed', 'offline');
    expect(c.state()).toBe('offline');
  });

  it('setOffline wins over a reconnecting socket but records offline transport', () => {
    const c = createConnection();
    c.setOffline();
    expect(c.state()).toBe('offline');
    expect(c.transport()).toBe('offline');
  });

  it('auth-expired outranks the socket lifecycle until a healthy open clears it', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.setAuthExpired();
    expect(c.state()).toBe('auth-expired');
    // A reconnecting socket must not hide the dead session.
    c.report('reconnecting', 'ws');
    expect(c.state()).toBe('auth-expired');
    // Only a fresh healthy connection clears it.
    c.report('open', 'ws');
    expect(c.state()).toBe('online');
  });

  it('auth-expired suppresses the offline signal', () => {
    const c = createConnection();
    c.setAuthExpired();
    c.setOffline();
    expect(c.state()).toBe('auth-expired');
  });
});
