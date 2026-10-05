import { describe, it, expect } from 'vitest';
import { createConnection } from './connection.ts';

describe('createConnection', () => {
  it('starts idle, not offline: nothing has asked for a connection yet', () => {
    const c = createConnection();
    expect(c.state()).toBe('idle');
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
    // A deliberate close (sign-out) ends the connection; it is not a loss.
    c.report('closed', 'offline');
    expect(c.state()).toBe('idle');
  });

  it('setOffline wins over a live socket and records offline transport', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.setOffline();
    expect(c.state()).toBe('offline');
    expect(c.transport()).toBe('offline');
    // The push client's next report is what clears it.
    c.report('open', 'ws');
    expect(c.state()).toBe('online');
  });

  it('ignores setOffline while idle (before start and after close)', () => {
    const c = createConnection();
    c.setOffline();
    expect(c.state()).toBe('idle');
    c.report('open', 'ws');
    c.report('closed', 'offline');
    c.setOffline();
    expect(c.state()).toBe('idle');
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
    c.report('open', 'ws');
    c.setAuthExpired();
    c.setOffline();
    expect(c.state()).toBe('auth-expired');
  });

  it('setReachable undoes setOffline by returning to the last push report', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.setOffline();
    expect(c.state()).toBe('offline');
    c.setReachable();
    expect(c.state()).toBe('online');
    expect(c.transport()).toBe('ws');
  });

  it('setReachable does not promote a socket that is itself down', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.report('reconnecting', 'ws');
    c.setReachable();
    expect(c.state()).toBe('connecting');

    // Network down, then the socket drops too: the drop is what stands.
    c.report('open', 'ws');
    c.setOffline();
    c.report('degraded', 'poll');
    c.setReachable();
    expect(c.state()).toBe('degraded');
    expect(c.transport()).toBe('poll');

    // Marked offline while degraded: reachable goes back to degraded, not online.
    c.setOffline();
    expect(c.state()).toBe('offline');
    c.setReachable();
    expect(c.state()).toBe('degraded');
  });

  it('setReachable does not lift auth-expired', () => {
    const c = createConnection();
    c.report('open', 'ws');
    c.setOffline();
    c.setAuthExpired();
    c.setReachable();
    expect(c.state()).toBe('auth-expired');
  });

  it('ignores setAuthExpired while idle, and a close ends an expiry', () => {
    const c = createConnection();
    c.setAuthExpired();
    expect(c.state()).toBe('idle');

    c.report('open', 'ws');
    c.setAuthExpired();
    expect(c.state()).toBe('auth-expired');
    c.report('closed', 'offline');
    expect(c.state()).toBe('idle');
    // The next session starts clean.
    c.report('connecting', 'ws');
    expect(c.state()).toBe('connecting');
  });
});
