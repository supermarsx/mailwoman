import { describe, it, expect, vi } from 'vitest';
import { changedTypes, createChangeReconciler } from './changes.ts';
import type { StateChange } from '../contracts/push.ts';

function change(changed: StateChange['changed']): StateChange {
  return { '@type': 'StateChange', changed };
}

describe('changedTypes', () => {
  it('reports every type present when there is no prior state', () => {
    const c = change({ a1: { Email: 'e1', Mailbox: 'm1' } });
    expect(changedTypes(undefined, c, 'a1').sort()).toEqual(['Email', 'Mailbox']);
  });

  it('reports only the types whose token advanced', () => {
    const c = change({ a1: { Email: 'e2', Mailbox: 'm1' } });
    expect(changedTypes({ Email: 'e1', Mailbox: 'm1' }, c, 'a1')).toEqual(['Email']);
  });

  it('returns nothing when the account is absent from the change', () => {
    const c = change({ a1: { Email: 'e1' } });
    expect(changedTypes(undefined, c, 'other')).toEqual([]);
  });

  it('ignores types the change does not mention', () => {
    const c = change({ a1: { Email: 'e2' } });
    expect(changedTypes({ Email: 'e1', Mailbox: 'm1' }, c, 'a1')).toEqual(['Email']);
  });
});

describe('createChangeReconciler', () => {
  it('fires the handler for the moved types on the first change', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.apply(change({ a1: { Email: 'e1', EmailSubmission: 's1' } }));
    expect(onChanged).toHaveBeenCalledTimes(1);
    expect(onChanged.mock.calls[0]![0]).toBe('a1');
    expect((onChanged.mock.calls[0]![1] as string[]).sort()).toEqual(['Email', 'EmailSubmission']);
  });

  it('does not re-fire for an unchanged, already-seen state', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.apply(change({ a1: { Email: 'e1' } }));
    r.apply(change({ a1: { Email: 'e1' } }));
    expect(onChanged).toHaveBeenCalledTimes(1);
  });

  it('fires again only for the type that advanced', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.apply(change({ a1: { Email: 'e1', Mailbox: 'm1' } }));
    onChanged.mockClear();
    r.apply(change({ a1: { Email: 'e2', Mailbox: 'm1' } }));
    expect(onChanged).toHaveBeenCalledTimes(1);
    expect(onChanged.mock.calls[0]![1]).toEqual(['Email']);
  });

  it('seed suppresses a refetch for the seeded state', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.seed('a1', { Email: 'e1' });
    r.apply(change({ a1: { Email: 'e1' } }));
    expect(onChanged).not.toHaveBeenCalled();
    r.apply(change({ a1: { Email: 'e2' } }));
    expect(onChanged).toHaveBeenCalledTimes(1);
  });

  it('tracks accounts independently', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.apply(change({ a1: { Email: 'e1' }, a2: { Email: 'x1' } }));
    expect(onChanged).toHaveBeenCalledTimes(2);
  });

  it('reset forgets all state so the next change refetches', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.apply(change({ a1: { Email: 'e1' } }));
    r.reset();
    onChanged.mockClear();
    r.apply(change({ a1: { Email: 'e1' } }));
    expect(onChanged).toHaveBeenCalledTimes(1);
  });
});
