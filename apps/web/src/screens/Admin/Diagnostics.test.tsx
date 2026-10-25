import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Diagnostics, type DiagnosticsApi } from './Diagnostics.tsx';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations } from './shell/uxTestkit.ts';

const result = { id: 'run-1', suite: 'server', startedAt: 1, status: 'fail', results: [{ id: 'db', summary: 'Database accepts a query', status: 'fail', durationMs: 5 }] };
describe('diagnostics console', () => {
  it('reports actual failed checks and unsaved history without claiming success', async () => {
    const api: DiagnosticsApi = {
      history: async () => ({ kind: 'ok', data: { runs: [], nextCursor: null }, etag: null }),
      run: vi.fn(async () => ({ run: result, historySaved: false, auditSaved: false })),
    };
    const { container } = renderInShell(() => <Diagnostics api={api} />);
    const button = await screen.findByRole('button', { name: 'Run server checks' });
    await vi.waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    expect(await screen.findByText('Database accepts a query')).toBeInTheDocument();
    expect(screen.getByRole('cell', { name: 'fail' })).toBeInTheDocument();
    expect(screen.getByText(/history or audit storage failed/)).toBeInTheDocument();
    expect(api.run).toHaveBeenCalledTimes(1);
    await expectNoAxeViolations(container);
  });
  it('loads saved history and follows its cursor', async () => {
    const history = vi.fn(async (cursor?: string) => ({ kind: 'ok' as const, etag: null, data: { runs: cursor ? [{ ...result, id: 'older', startedAt: 0 }] : [result], nextCursor: cursor ? null : '1:run-1' } }));
    renderInShell(() => <Diagnostics api={{ history, run: async () => ({ run: result, historySaved: true, auditSaved: true }) }} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Older runs' }));
    await vi.waitFor(() => expect(history).toHaveBeenLastCalledWith('1:run-1'));
  });
});
