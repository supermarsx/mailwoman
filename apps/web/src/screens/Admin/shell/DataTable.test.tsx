import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, within } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { DataTable, type Column, type SortState } from './DataTable.tsx';

interface Row {
  name: string;
  users: number;
}
const rows: Row[] = [
  { name: 'a.example', users: 3 },
  { name: 'b.example', users: 1 },
  { name: 'c.example', users: 0 },
];
const columns: Column<Row>[] = [
  { id: 'name', header: 'Domain', cell: (r) => r.name, sortable: true },
  { id: 'users', header: 'Users', cell: (r) => String(r.users) },
];

describe('DataTable', () => {
  it('renders rows under a named table with column headers', () => {
    render(() => <DataTable caption="Domains" columns={columns} rows={rows} rowKey={(r) => r.name} state="ready" emptyText="none" />);
    const table = screen.getByRole('table', { name: 'Domains' });
    expect(within(table).getAllByRole('columnheader').map((h) => h.textContent)).toEqual(['Domain', 'Users']);
    expect(within(table).getAllByRole('row')).toHaveLength(4);
  });

  it('loading and error states, with retry', () => {
    const retry = vi.fn();
    const [state, setState] = createSignal<'loading' | 'ready' | 'error'>('loading');
    render(() => (
      <DataTable caption="D" columns={columns} rows={[]} rowKey={(r) => r.name} state={state()} emptyText="none" onRetry={retry} errorText="Could not load domains" />
    ));
    expect(screen.getByRole('status')).toHaveTextContent('Loading');
    setState('error');
    expect(screen.getByRole('alert')).toHaveTextContent('Could not load domains');
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(retry).toHaveBeenCalled();
    expect(screen.queryByRole('table')).toBeNull();
  });

  it('the empty state names the next action and runs it', () => {
    const add = vi.fn();
    render(() => (
      <DataTable caption="D" columns={columns} rows={[]} rowKey={(r) => r.name} state="ready" emptyText="No domains yet." emptyAction={{ label: 'Add a domain', onClick: add }} />
    ));
    expect(screen.getByTestId('table-empty')).toHaveTextContent('No domains yet.');
    fireEvent.click(screen.getByRole('button', { name: 'Add a domain' }));
    expect(add).toHaveBeenCalled();
  });

  it('sortable headers are buttons and report aria-sort', () => {
    const [sort, setSort] = createSignal<SortState | null>(null);
    const onSort = vi.fn((col: string) => setSort({ column: col, direction: sort()?.direction === 'asc' ? 'desc' : 'asc' }));
    render(() => <DataTable caption="D" columns={columns} rows={rows} rowKey={(r) => r.name} state="ready" emptyText="none" sort={sort()} onSort={onSort} />);
    const th = screen.getAllByRole('columnheader')[0]!;
    expect(th).toHaveAttribute('aria-sort', 'none');
    fireEvent.click(within(th).getByRole('button', { name: /Domain/ }));
    expect(onSort).toHaveBeenCalledWith('name');
    expect(th).toHaveAttribute('aria-sort', 'ascending');
    fireEvent.click(within(th).getByRole('button'));
    expect(th).toHaveAttribute('aria-sort', 'descending');
    // A non-sortable header is plain text.
    expect(within(screen.getAllByRole('columnheader')[1]!).queryByRole('button')).toBeNull();
  });

  it('rows are one Tab stop; arrow keys, Home and End move between them', () => {
    render(() => <DataTable caption="D" columns={columns} rows={rows} rowKey={(r) => r.name} state="ready" emptyText="none" />);
    const body = screen.getAllByRole('row').slice(1);
    expect(body.map((r) => r.getAttribute('tabindex'))).toEqual(['0', '-1', '-1']);
    body[0]!.focus();
    fireEvent.keyDown(body[0]!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(body[1]);
    fireEvent.keyDown(body[1]!, { key: 'End' });
    expect(document.activeElement).toBe(body[2]);
    fireEvent.keyDown(body[2]!, { key: 'Home' });
    expect(document.activeElement).toBe(body[0]);
    expect(body.map((r) => r.getAttribute('tabindex'))).toEqual(['0', '-1', '-1']);
  });

  it('search reports to the server after a pause, and at once on Enter', () => {
    vi.useFakeTimers();
    const onSearch = vi.fn();
    render(() => (
      <DataTable caption="D" columns={columns} rows={rows} rowKey={(r) => r.name} state="ready" emptyText="none" search={{ label: 'Search domains', value: '', onSearch }} />
    ));
    const box = screen.getByRole('searchbox', { name: 'Search domains' });
    fireEvent.input(box, { target: { value: 'ex' } });
    expect(onSearch).not.toHaveBeenCalled();
    vi.advanceTimersByTime(300);
    expect(onSearch).toHaveBeenCalledWith('ex');
    fireEvent.input(box, { target: { value: 'exa' } });
    fireEvent.submit(screen.getByRole('search'));
    expect(onSearch).toHaveBeenLastCalledWith('exa');
    vi.advanceTimersByTime(300);
    expect(onSearch).toHaveBeenCalledTimes(2);
    vi.useRealTimers();
  });

  it('filter chips are toggle buttons; paging buttons follow the cursor state', () => {
    const toggle = vi.fn();
    const next = vi.fn();
    render(() => (
      <DataTable
        caption="Users"
        columns={columns}
        rows={rows}
        rowKey={(r) => r.name}
        state="ready"
        emptyText="none"
        filters={[{ id: 'disabled', label: 'Disabled', active: true }, { id: 'zero', label: 'Zero-access', active: false }]}
        onToggleFilter={toggle}
        paging={{ hasPrev: false, hasNext: true, onPrev: () => undefined, onNext: next }}
      />
    ));
    expect(screen.getByRole('button', { name: 'Disabled' })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: 'Zero-access' }));
    expect(toggle).toHaveBeenCalledWith('zero');
    const pager = screen.getByRole('navigation', { name: 'Pages of Users' });
    expect(within(pager).getByRole('button', { name: 'Previous page' })).toBeDisabled();
    fireEvent.click(within(pager).getByRole('button', { name: 'Next page' }));
    expect(next).toHaveBeenCalled();
  });

  it('row actions get their own column', () => {
    render(() => (
      <DataTable caption="D" columns={columns} rows={rows} rowKey={(r) => r.name} state="ready" emptyText="none" rowActions={(r) => <button type="button">Delete {r.name}</button>} />
    ));
    expect(screen.getAllByRole('columnheader').at(-1)).toHaveTextContent('Actions');
    expect(screen.getByRole('button', { name: 'Delete b.example' })).toBeInTheDocument();
  });
});
