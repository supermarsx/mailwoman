import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AuditRetention, type RetentionApi, type RetentionState } from './AuditRetention.tsx';
import { AdminProblemError } from '../shell/adminApi.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations } from '../shell/uxTestkit.ts';

function state(over: Partial<RetentionState> = {}): RetentionState {
  return {
    categories: ['*', 'admin', 'security', 'alert'],
    rows: [],
    bounds: { minDays: 30, maxDays: 3650 },
    lastPrune: null,
    task: null,
    ...over,
  };
}

function api(initial: RetentionState): RetentionApi & { calls: string[] } {
  let s = initial;
  const calls: string[] = [];
  return {
    calls,
    load: vi.fn(async () => s),
    put: vi.fn(async (category, body) => {
      calls.push(`put:${category}:${JSON.stringify(body)}`);
      s = { ...s, rows: [...s.rows.filter((r) => r.category !== category), { category, days: body.days, legalHold: body.legalHold, updatedBy: 'root', updatedAt: 1 }], task: { id: 't', enabled: true, nextRunAt: 1_900_000_000 } };
      return s;
    }),
    reset: vi.fn(async (category) => {
      calls.push(`reset:${category}`);
      s = { ...s, rows: s.rows.filter((r) => r.category !== category) };
      return s;
    }),
    prune: vi.fn(async () => {
      calls.push('prune');
      s = { ...s, lastPrune: { ts: 1_800_000_000 } };
      return { ...s, report: { total: 3, removed: { admin: 3 } } };
    }),
  };
}

const row = (category: string): HTMLElement => document.querySelector(`[data-category="${category}"]`) as HTMLElement;

describe('Admin → audit log retention', () => {
  it('keeps every entry by default and saves a bounded default', async () => {
    const a = api(state());
    const onChanged = vi.fn();
    const { container } = renderInShell(() => <AuditRetention api={a} canWrite onChanged={onChanged} />);
    await waitFor(() => expect(row('*')).toBeTruthy());
    expect(row('*').querySelector('[data-kept]')?.textContent).toBe('Every entry');
    expect(row('security').querySelector('[data-kept]')?.textContent).toBe('Default: Every entry');
    expect(screen.getByText(/No prune task yet/)).toBeInTheDocument();
    fireEvent.change(within(row('*')).getByLabelText('Keep'), { target: { value: 'days' } });
    const days = within(row('*')).getByLabelText('Days');
    fireEvent.input(days, { target: { value: '10' } });
    expect(within(row('*')).getByText(/Enter a whole number from 30 to 3.?650/)).toBeInTheDocument();
    expect(within(row('*')).getByRole('button', { name: /Save/ })).toBeDisabled();
    fireEvent.input(days, { target: { value: '90' } });
    fireEvent.click(within(row('*')).getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(a.calls).toEqual(['put:*:{"days":90,"legalHold":false}']));
    await waitFor(() => expect(row('*').querySelector('[data-kept]')?.textContent).toBe('90 days'));
    expect(row('admin').querySelector('[data-kept]')?.textContent).toBe('Default: 90 days');
    expect(screen.getByText(/Next scheduled prune/)).toBeInTheDocument();
    expect(onChanged).toHaveBeenCalled();
    await expectNoAxeViolations(container);
  });

  it('sets a legal hold, resets a category to the default, and prunes after confirmation', async () => {
    const a = api(state({ rows: [{ category: 'security', days: 365, legalHold: false, updatedBy: 'root', updatedAt: 1 }] }));
    renderInShell(() => <AuditRetention api={a} canWrite />);
    await waitFor(() => expect(row('security').querySelector('[data-kept]')?.textContent).toBe('365 days'));
    fireEvent.click(within(row('security')).getByRole('checkbox', { name: 'Legal hold' }));
    fireEvent.click(within(row('security')).getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(a.calls).toContain('put:security:{"days":365,"legalHold":true}'));
    await waitFor(() => expect(row('security').querySelector('[data-hold]')).not.toBeNull());
    fireEvent.change(within(row('security')).getByLabelText('Keep'), { target: { value: 'inherit' } });
    fireEvent.click(within(row('security')).getByRole('button', { name: /Save/ }));
    await waitFor(() => expect(a.calls).toContain('reset:security'));
    fireEvent.click(screen.getByRole('button', { name: 'Prune now' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(a.calls).not.toContain('prune');
    fireEvent.click(within(dialog).getByRole('button', { name: 'Prune now' }));
    await waitFor(() => expect(a.calls).toContain('prune'));
    expect(await screen.findByText('3 entries removed.')).toBeInTheDocument();
    expect(screen.getByText(/Last prune that removed entries/)).toBeInTheDocument();
  });

  it('is read-only without settings.write and hidden from a scoped auditor', async () => {
    const a = api(state());
    renderInShell(() => <AuditRetention api={a} canWrite={false} />);
    await waitFor(() => expect(row('*')).toBeTruthy());
    expect(within(row('*')).queryByRole('button')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Prune now' })).toBeNull();
    const scoped: RetentionApi = { ...a, load: async () => { throw new AdminProblemError(403, { code: 'outOfScope' }, null); } };
    const second = renderInShell(() => <AuditRetention api={scoped} />);
    await waitFor(() => expect(second.container.querySelector('[data-testid="audit-retention"]')).toBeNull());
  });
});
