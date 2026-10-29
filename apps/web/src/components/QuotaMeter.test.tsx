import { describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@solidjs/testing-library';
import { createSignal, Suspense } from 'solid-js';
import type { Quota } from '../api/quota.ts';
import { loadQuotas, storageQuota, warnLevel, warnPolicy } from '../api/quota.ts';
import type { Client } from '../api/client.ts';

const state = vi.hoisted(() => ({ read: (): Record<string, { value: unknown }> => ({}) }));
vi.mock('../api/settings.ts', () => ({ settings: () => state.read() }));

import { QuotaDetails, QuotaMeter } from './QuotaMeter.tsx';

const MiB = 1024 * 1024;
const storage = (used: number, limit: number, id = 'upstream-storage'): Quota =>
  ({ id, resourceType: 'octets', used, hardLimit: limit, name: 'User quota' });

describe('quota model', () => {
  it('picks the fullest storage quota and ignores counts', () => {
    const list = [storage(10, 100, 'a'), { ...storage(9, 10, 'b'), resourceType: 'count' }, storage(50, 100, 'c')];
    expect(storageQuota(list)?.id).toBe('c');
    expect(storageQuota([])).toBeNull();
  });
  it('warns at the configured thresholds and not when warnings are off', () => {
    const policy = warnPolicy({ 'quota.warnEnabled': { value: true, source: 'default', locked: true }, 'quota.warnThresholds': { value: [95, 80], source: 'default', locked: true } });
    expect(policy.thresholds).toEqual([80, 95]);
    expect(warnLevel(storage(79, 100), policy)).toBeNull();
    expect(warnLevel(storage(80, 100), policy)).toEqual({ level: 'warn', threshold: 80 });
    expect(warnLevel(storage(96, 100), policy)).toEqual({ level: 'critical', threshold: 95 });
    expect(warnLevel(storage(99, 100), { enabled: false, thresholds: [80, 95] })).toBeNull();
    expect(warnLevel(storage(60, 100), warnPolicy({ 'quota.warnThresholds': { value: [50], source: 'global', locked: true } })))
      .toEqual({ level: 'warn', threshold: 50 });
    expect(warnPolicy({}).thresholds).toEqual([80, 95]);
  });
  it('reports nothing when the session lacks the quota capability or the method is unknown', async () => {
    const noCap = { session: vi.fn(async () => ({ capabilities: {}, accounts: { a: { accountCapabilities: {} } } })), jmap: vi.fn() } as unknown as Client;
    expect(await loadQuotas(noCap, 'a')).toEqual([]);
    expect(noCap.jmap).not.toHaveBeenCalled();
    const cap = { 'urn:ietf:params:jmap:quota': {} };
    const unknown = { session: vi.fn(async () => ({ capabilities: cap, accounts: { a: { accountCapabilities: cap } } })),
      jmap: vi.fn(async () => ({ methodResponses: [['error', { type: 'unknownMethod' }, 'q']] })) } as unknown as Client;
    expect(await loadQuotas(unknown, 'a')).toEqual([]);
    const some = { session: unknown.session,
      jmap: vi.fn(async () => ({ methodResponses: [['Quota/get', { list: [storage(1, 2), storage(1, 0, 'unlimited')] }, 'q']] })) } as unknown as Client;
    expect((await loadQuotas(some, 'a')).map((q) => q.id)).toEqual(['upstream-storage']);
  });
});

describe('QuotaMeter', () => {
  // t60: the meter sits in the mailbox shell, under the route's <Suspense>. A
  // first load that suspends it swaps the whole shell out and back, which
  // reloads every open message frame.
  it('never suspends the surrounding shell while the quota loads', async () => {
    state.read = () => ({});
    let release: () => void = () => {};
    const gate = new Promise<void>((resolve) => { release = resolve; });
    const load = vi.fn(async () => { await gate; return [storage(10 * MiB, 100 * MiB)]; });
    render(() => <Suspense fallback={<p>Shell loading</p>}><p>Shell</p><QuotaMeter accountId="a" load={load} /></Suspense>);
    await waitFor(() => expect(load).toHaveBeenCalled());
    expect(screen.getByText('Shell')).toBeInTheDocument();
    expect(screen.queryByText('Shell loading')).toBeNull();
    release();
    expect(await screen.findByRole('meter', { name: 'Mailbox storage' })).toBeInTheDocument();
    expect(screen.queryByText('Shell loading')).toBeNull();
  });
  it('shows nothing when the server has no quota', async () => {
    const load = vi.fn(async () => [] as Quota[]);
    const { container } = render(() => <QuotaMeter accountId="a" load={load} />);
    await waitFor(() => expect(load).toHaveBeenCalledWith('a'));
    expect(container).toBeEmptyDOMElement();
  });
  it('shows usage, then a warning past a threshold, and reloads on refresh', async () => {
    state.read = () => ({});
    let used = 10 * MiB;
    const load = vi.fn(async () => [storage(used, 100 * MiB)]);
    const [tick, setTick] = createSignal(0);
    render(() => <QuotaMeter accountId="a" refresh={tick()} load={load} />);
    const meter = await screen.findByRole('meter', { name: 'Mailbox storage' });
    expect(meter).toHaveAttribute('aria-valuenow', '10');
    expect(screen.getByText('10 MB of 100 MB used')).toBeInTheDocument();
    expect(screen.getByTestId('quota-meter')).toHaveAttribute('data-level', 'ok');
    expect(screen.queryByRole('status')).toBeNull();

    used = 85 * MiB;
    setTick(1);
    await waitFor(() => expect(screen.getByTestId('quota-meter')).toHaveAttribute('data-level', 'warn'));
    expect(screen.getByRole('status')).toHaveTextContent('Mailbox 85% full');

    used = 97 * MiB;
    setTick(2);
    await waitFor(() => expect(screen.getByTestId('quota-meter')).toHaveAttribute('data-level', 'critical'));
    expect(screen.getByRole('status')).toHaveTextContent('Delete or archive mail');
  });
  it('respects quota.warnEnabled = false', async () => {
    state.read = () => ({ 'quota.warnEnabled': { value: false } });
    render(() => <QuotaMeter accountId="a" load={async () => [storage(99, 100)]} />);
    await screen.findByRole('meter');
    expect(screen.getByTestId('quota-meter')).toHaveAttribute('data-level', 'ok');
    expect(screen.queryByRole('status')).toBeNull();
    state.read = () => ({});
  });
  it('a failed read shows nothing rather than an error', async () => {
    const load = vi.fn(async () => { throw new Error('boom'); });
    const { container } = render(() => <QuotaMeter accountId="a" load={load} />);
    await waitFor(() => expect(load).toHaveBeenCalled());
    expect(container).toBeEmptyDOMElement();
  });
});

describe('QuotaDetails', () => {
  it('lists every quota with its source', async () => {
    render(() => <QuotaDetails accountId="a" load={async () => [
      storage(2 * MiB, 4 * MiB),
      { id: 'mail-count', resourceType: 'count', used: 12, hardLimit: 1000 },
    ]} />);
    const table = await screen.findByRole('table');
    expect(table).toHaveTextContent('2 MB of 4 MB (50%)');
    expect(table).toHaveTextContent('12 of 1,000 (1%)');
    expect(screen.getByRole('rowheader', { name: 'Storage' })).toBeInTheDocument();
    expect(screen.getByRole('rowheader', { name: 'Messages' })).toBeInTheDocument();
    expect(table).toHaveTextContent('Your mail server');
    expect(table).toHaveTextContent('Your administrator');
  });
  it('renders nothing without a quota', async () => {
    const load = vi.fn(async () => [] as Quota[]);
    const { container } = render(() => <QuotaDetails accountId="a" load={load} />);
    await waitFor(() => expect(load).toHaveBeenCalled());
    expect(container).toBeEmptyDOMElement();
  });
});
