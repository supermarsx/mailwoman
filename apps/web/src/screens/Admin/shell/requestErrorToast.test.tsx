// t104: a failed admin request is the corner toast with Retry, and a short 429
// is retried by itself for an idempotent request with no toast at all.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, within } from '@solidjs/testing-library';
import { createSignal, Show } from 'solid-js';
import { showAdminError, clearAdminError } from './requestErrorToast.ts';
import { renderInShell } from './testkit.tsx';
import { AdminProblemError, adminRequest } from './adminApi.ts';
import { describeAdminError } from './Feedback.tsx';
import { resetToasts } from '../../../components/toast/index.ts';

afterEach(() => { vi.unstubAllGlobals(); vi.useRealTimers(); resetToasts(); });

function limited(seconds: number): AdminProblemError {
  return new AdminProblemError(429, { code: 'rate_limited', detail: 'Too many requests.', retryAfterSecs: seconds }, null, String(seconds));
}

/**
 * A stand-in screen, independent of any real screen's controls: one action
 * button that runs `action` and reports failures the way the console screens
 * do (showAdminError, inline fallback).
 */
function Probe(props: { action: () => Promise<void> }) {
  const [inline, setInline] = createSignal<string>();
  const [done, setDone] = createSignal(false);
  const go = async (): Promise<void> => {
    try { await props.action(); clearAdminError('probe'); setDone(true); } catch (e) { showAdminError(e, { key: 'probe', retry: go }, setInline); }
  };
  return (
    <section aria-label="Probe">
      <button type="button" onClick={() => void go()}>Do it</button>
      <Show when={inline()}><p role="alert">{inline()}</p></Show>
      <Show when={done()}><p>Done</p></Show>
    </section>
  );
}

describe('admin request errors (t104)', () => {
  it('a 429 is the corner toast with the wait and a Retry that waits it out, never an inline line', async () => {
    const action = vi.fn<() => Promise<void>>().mockRejectedValueOnce(limited(1)).mockResolvedValue(undefined);
    renderInShell(() => <Probe action={action} />);
    fireEvent.click(screen.getByRole('button', { name: 'Do it' }));
    const toast = await screen.findByTestId('admin-request-error');
    expect(within(screen.getByTestId('toast-region')).getByTestId('admin-request-error')).toBe(toast);
    expect(toast).toHaveTextContent('Too many requests, try again in 1 s.');
    expect(within(screen.getByRole('region', { name: 'Probe' })).queryByRole('alert')).not.toBeInTheDocument();
    const started = Date.now();
    fireEvent.click(within(toast).getByRole('button', { name: 'Retry' }));
    await vi.waitFor(() => expect(action).toHaveBeenCalledTimes(2), { timeout: 3000 });
    expect(Date.now() - started).toBeGreaterThanOrEqual(900);
    expect(await screen.findByText('Done')).toBeInTheDocument();
    expect(screen.queryByTestId('admin-request-error')).not.toBeInTheDocument();
  });

  it('any failed request is a toast with Retry; a 422 naming a field stays inline', async () => {
    const action = vi.fn<() => Promise<void>>()
      .mockRejectedValueOnce(new AdminProblemError(503, { code: 'x' }, null))
      .mockRejectedValueOnce(new AdminProblemError(422, { code: 'invalidValue', field: '/port' }, null))
      .mockResolvedValue(undefined);
    renderInShell(() => <Probe action={action} />);
    fireEvent.click(screen.getByRole('button', { name: 'Do it' }));
    const toast = await screen.findByTestId('admin-request-error');
    fireEvent.click(within(toast).getByRole('button', { name: 'Retry' }));
    await vi.waitFor(() => expect(action).toHaveBeenCalledTimes(2));
    expect(await within(screen.getByRole('region', { name: 'Probe' })).findByRole('alert')).toBeInTheDocument();
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
