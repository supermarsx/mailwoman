import { afterEach, expect, it, vi } from 'vitest';
import { createClient } from './client.ts';
import { LOGIN_TIMEOUT_MS, LoginTimeoutError } from './loginTimeout.ts';

afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });
it('aborts a hanging login once, prevents concurrent submissions, and never retries', async () => {
  vi.useFakeTimers();
  let signal: AbortSignal | null | undefined;
  const fetcher = vi.fn((_url: unknown, init?: RequestInit) => { signal = init?.signal; return new Promise<Response>(() => undefined); });
  vi.stubGlobal('fetch', fetcher);
  const client = createClient('');
  const first = client.login({ username: 'alice@example.test', password: 'wrong' });
  const assertion = expect(first).rejects.toBeInstanceOf(LoginTimeoutError);
  await expect(client.login({ username: 'alice@example.test', password: 'wrong' })).rejects.toThrow('already in progress');
  await vi.advanceTimersByTimeAsync(LOGIN_TIMEOUT_MS);
  await assertion;
  expect(signal?.aborted).toBe(true);
  await vi.advanceTimersByTimeAsync(120_000);
  expect(fetcher).toHaveBeenCalledTimes(1);
});
it('does not retry rejected credentials and permits a deliberate new attempt', async () => {
  const fetcher = vi.fn(async () => new Response('{"error":"invalid credentials"}', { status: 401 }));
  vi.stubGlobal('fetch', fetcher);
  const client = createClient('');
  await expect(client.login({ username: 'alice', password: 'wrong' })).rejects.toMatchObject({ status: 401 });
  expect(fetcher).toHaveBeenCalledTimes(1);
  await expect(client.login({ username: 'alice', password: 'new attempt' })).rejects.toMatchObject({ status: 401 });
  expect(fetcher).toHaveBeenCalledTimes(2);
});
