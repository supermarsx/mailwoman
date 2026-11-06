import { describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import Restrictions, { type RestrictionRow, type RestrictionsApi } from './index.tsx';
import { COMPLIANCE_TABS } from '../Compliance/tabs.ts';
import { renderInShell } from '../shell/testkit.tsx';

const row = (over: Partial<RestrictionRow> = {}): RestrictionRow => ({
  id: 'r1', accountId: 'a1', address: 'ana@example.test', domain: 'example.test', scopes: ['assistant', 'push'],
  mailDelivery: 'keep', reason: 'dispute', source: 'dsr', requestId: 'abcdef12-0000', createdBy: 'admin', createdAt: 1_700_000_000,
  expiresAt: null, liftedAt: null, liftedBy: null, liftReason: null, active: true, ...over,
});
const api = (over: Partial<RestrictionsApi> = {}): RestrictionsApi => ({
  list: vi.fn(async () => ({ restrictions: [row()], scopes: ['indexing', 'assistant', 'analytics', 'classification', 'push', 'thirdParty'], mailDelivery: ['keep', 'queue', 'reject'] })),
  apply: vi.fn(async () => {}),
  lift: vi.fn(async () => {}),
  settings: vi.fn(async () => []),
  ...over,
});

describe('Admin restrictions', () => {
  it('is registered in the Compliance hub with its own tab id', () => {
    expect(COMPLIANCE_TABS.some((t) => t.id === 'restrictions')).toBe(true);
  });

  it('lists restricted accounts with scopes, source and request', async () => {
    renderInShell(() => <Restrictions api={api()} />);
    const r = await screen.findByTestId('restriction-r1');
    expect(r).toHaveTextContent('ana@example.test');
    expect(r).toHaveTextContent('Assistant, Push notifications');
    expect(r).toHaveTextContent('data request');
    expect(r).toHaveTextContent('until lifted');
    cleanup();
  });

  it('applies a restriction with a reason and the chosen scopes', async () => {
    const a = api();
    renderInShell(() => <Restrictions api={a} />);
    await screen.findByTestId('restriction-table');
    fireEvent.input(screen.getByLabelText('Address'), { target: { value: 'bo@example.test' } });
    fireEvent.input(screen.getByLabelText('Reason'), { target: { value: 'contests accuracy' } });
    fireEvent.click(screen.getByLabelText('Analytics and open tracking'));
    fireEvent.click(screen.getByTestId('restriction-apply'));
    await waitFor(() => expect(a.apply).toHaveBeenCalledTimes(1));
    const body = vi.mocked(a.apply).mock.calls[0]![0];
    expect(body.address).toBe('bo@example.test');
    expect(body.reason).toBe('contests accuracy');
    expect(body.scopes).not.toContain('analytics');
    expect(body.scopes).toContain('assistant');
    cleanup();
  });

  it('lifts with a reason and refuses an empty reason', async () => {
    const a = api();
    const prompt = vi.spyOn(window, 'prompt');
    renderInShell(() => <Restrictions api={a} />);
    prompt.mockReturnValueOnce('  ');
    fireEvent.click(await screen.findByTestId('restriction-lift-r1'));
    expect(a.lift).not.toHaveBeenCalled();
    prompt.mockReturnValueOnce('dispute settled');
    fireEvent.click(within(await screen.findByTestId('restriction-r1')).getByTestId('restriction-lift-r1'));
    await waitFor(() => expect(a.lift).toHaveBeenCalledWith('ana@example.test', 'dispute settled'));
    prompt.mockRestore();
    cleanup();
  });

  it('shows a lifted restriction without a lift button', async () => {
    renderInShell(() => <Restrictions api={api({ list: vi.fn(async () => ({ restrictions: [row({ active: false, liftedAt: 1_700_100_000, liftReason: 'settled' })], scopes: ['assistant'], mailDelivery: [] })) })} />);
    const r = await screen.findByTestId('restriction-r1');
    expect(r).toHaveTextContent('settled');
    expect(within(r).queryByTestId('restriction-lift-r1')).toBeNull();
    cleanup();
  });
});
