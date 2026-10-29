// t104: a failed admin request is the corner toast with Retry, and a short 429
// is retried by itself for an idempotent request with no toast at all.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, within } from '@solidjs/testing-library';
import { Diagnostics, type DiagnosticsApi } from '../Diagnostics.tsx';
import { renderInShell } from './testkit.tsx';
import { AdminProblemError, adminRequest } from './adminApi.ts';
import { describeAdminError } from './Feedback.tsx';
import { resetToasts } from '../../../components/toast/index.ts';

afterEach(() => { vi.unstubAllGlobals(); vi.useRealTimers(); resetToasts(); });

function limited(seconds: number): AdminProblemError {
  return new AdminProblemError(429, { code: 'rate_limited', detail: 'Too many requests.', retryAfterSecs: seconds }, null, String(seconds));
}

const empty = { kind: 'ok' as const, data: { runs: [], nextCursor: null }, etag: null };
const result = { id: 'run-1', suite: 'server', startedAt: 1, status: 'pass', results: [{ id: 'db', summary: 'Database accepts a query', status: 'pass', durationMs: 5 }] };

describe('admin request errors (t104)', () => {
  it('a 429 is the corner toast with the wait and a Retry that waits it out, never an inline line', async () => {
    let calls = 0;
    const run = vi.fn(async () => {
      calls += 1;
      if (calls === 1) throw limited(1);
      return { run: result, historySaved: true, auditSaved: true };
    });
    const api: DiagnosticsApi = { history: async () => empty, run };
    renderInShell(() => <Diagnostics api={api} />);
    const button = await screen.findByRole('button', { name: 'Run server checks' });
    await vi.waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    const toast = await screen.findByTestId('admin-request-error');
    expect(within(screen.getByTestId('toast-region')).getByTestId('admin-request-error')).toBe(toast);
    expect(toast).toHaveTextContent('Too many requests, try again in 1 s.');
    const section = screen.getByRole('region', { name: 'Diagnostics' });
    expect(within(section).queryByRole('alert')).not.toBeInTheDocument();
    const started = Date.now();
    fireEvent.click(within(toast).getByRole('button', { name: 'Retry' }));
    await vi.waitFor(() => expect(run).toHaveBeenCalledTimes(2), { timeout: 3000 });
    expect(Date.now() - started).toBeGreaterThanOrEqual(900);
    expect(await screen.findByText('Database accepts a query')).toBeInTheDocument();
    expect(screen.queryByTestId('admin-request-error')).not.toBeInTheDocument();
  });

  it('a failed backends job start is a toast too, and Retry starts it again', async () => {
    const job = { run: { ...result, id: 'bulk', status: 'pass' }, total: 1, completed: 1, limited: false, historySaved: true };
    const start = vi.fn<() => Promise<typeof job>>().mockRejectedValueOnce(new AdminProblemError(503, { code: 'x' }, null)).mockResolvedValue(job);
    renderInShell(() => <Diagnostics api={{ history: async () => empty, run: async () => ({ run: result, historySaved: true, auditSaved: true }), start, job: async () => job }} />);
    const button = await screen.findByRole('button', { name: 'Check all saved backends' });
    await vi.waitFor(() => expect(button).toBeEnabled());
    fireEvent.click(button);
    const toast = await screen.findByTestId('admin-request-error');
    fireEvent.click(within(toast).getByRole('button', { name: 'Retry' }));
    await vi.waitFor(() => expect(start).toHaveBeenCalledTimes(2));
  });

  it('describes any 429 that names its wait', () => {
    expect(describeAdminError(limited(12))).toBe('Too many requests, try again in 12 s.');
    expect(describeAdminError({ status: 429 })).toBe('Too many requests. Wait a minute and try again.');
  });

  it('adminRequest waits out a short Retry-After once for an idempotent request', async () => {
    const fetch = vi.fn<(url: string, init?: RequestInit) => Promise<Response>>()
      .mockResolvedValueOnce(new Response('{"code":"rate_limited","retryAfterSecs":1}', { status: 429, headers: { 'Retry-After': '1', 'Content-Type': 'application/json' } }))
      .mockResolvedValue(new Response('{"ok":true}', { status: 200, headers: { 'Content-Type': 'application/json' } }));
    vi.stubGlobal('fetch', fetch);
    vi.useFakeTimers();
    const pending = adminRequest('/diag/jobs', { method: 'POST', body: { suite: 'backends' }, idempotent: true, base: '' });
    await vi.advanceTimersByTimeAsync(1000);
    await expect(pending).resolves.toMatchObject({ status: 200, data: { ok: true } });
    expect(fetch).toHaveBeenCalledTimes(2);
  });

  it('does not resend a non-idempotent POST or wait out a long Retry-After', async () => {
    const answer = (after: string): Response => new Response('{"code":"rate_limited"}', { status: 429, headers: { 'Retry-After': after, 'Content-Type': 'application/json' } });
    const fetch = vi.fn<(url: string, init?: RequestInit) => Promise<Response>>(async () => answer('1'));
    vi.stubGlobal('fetch', fetch);
    const err = await adminRequest('/users', { method: 'POST', body: {}, base: '' }).catch((e: unknown) => e);
    expect(err).toMatchObject({ status: 429, retryAfterSecs: 1 });
    expect(fetch).toHaveBeenCalledTimes(1);
    fetch.mockImplementation(async () => answer('60'));
    const long = await adminRequest('/settings', { base: '' }).catch((e: unknown) => e);
    expect(long).toMatchObject({ status: 429, retryAfterSecs: 60 });
    expect(fetch).toHaveBeenCalledTimes(2);
  });
});
