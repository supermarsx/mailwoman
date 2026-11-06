import { describe, expect, it, vi } from 'vitest';
import { ApiError, NetworkError } from '../api/client.ts';
import { bootReconnecting, isServerUnreachable, retryWhileUnreachable } from './bootRetry.ts';
import { createChangeReconciler } from '../realtime/changes.ts';
import { createRealtimeController } from '../realtime/controller.ts';

const instant = (): Promise<void> => Promise.resolve();

describe('retryWhileUnreachable', () => {
  it('waits out a restart: 502, 503 and a refused connection are not a sign-out', async () => {
    const attempt = vi.fn()
      .mockRejectedValueOnce(new ApiError(502, 'bad gateway'))
      .mockRejectedValueOnce(new ApiError(503, 'unavailable'))
      .mockRejectedValueOnce(new NetworkError('connection refused'))
      .mockResolvedValueOnce({ username: 'a@example.org' });
    const seen: boolean[] = [];
    const sleep = vi.fn(async () => { seen.push(bootReconnecting()); });
    await expect(retryWhileUnreachable(attempt, { sleep })).resolves.toEqual({ username: 'a@example.org' });
    expect(attempt).toHaveBeenCalledTimes(4);
    expect(seen).toEqual([true, true, true]);
    expect(bootReconnecting()).toBe(false);
  });

  it('a 401 is the signed-out answer and is never retried', async () => {
    const attempt = vi.fn().mockRejectedValue(new ApiError(401, 'not authenticated'));
    await expect(retryWhileUnreachable(attempt, { sleep: instant })).rejects.toMatchObject({ status: 401 });
    expect(attempt).toHaveBeenCalledTimes(1);
    expect(bootReconnecting()).toBe(false);
  });

  it('other refusals are not retried either', async () => {
    const attempt = vi.fn().mockRejectedValue(new ApiError(500, 'boom'));
    await expect(retryWhileUnreachable(attempt, { sleep: instant })).rejects.toMatchObject({ status: 500 });
    expect(attempt).toHaveBeenCalledTimes(1);
  });

  it('gives up after the last attempt so the boot card can offer Retry', async () => {
    const attempt = vi.fn().mockRejectedValue(new ApiError(503, 'unavailable'));
    await expect(retryWhileUnreachable(attempt, { attempts: 3, sleep: instant })).rejects.toMatchObject({ status: 503 });
    expect(attempt).toHaveBeenCalledTimes(3);
    expect(bootReconnecting()).toBe(false);
  });

  it('classifies unreachable errors', () => {
    expect(isServerUnreachable(new NetworkError('x'))).toBe(true);
    expect(isServerUnreachable(new ApiError(504, 'x'))).toBe(true);
    expect(isServerUnreachable(new ApiError(401, 'x'))).toBe(false);
    expect(isServerUnreachable(new Error('x'))).toBe(false);
  });
});

describe('resync after the stream returns', () => {
  it('re-reads every known account once', () => {
    const onChanged = vi.fn();
    const r = createChangeReconciler(onChanged);
    r.seed('acc', { Email: '1' });
    r.resync();
    expect(onChanged).toHaveBeenCalledWith('acc', ['Email', 'Mailbox', 'EmailSubmission', 'Thread']);
  });

  it('the controller resyncs on reconnecting -> open, not on a first open', () => {
    vi.useFakeTimers();
    try {
      const onChanged = vi.fn();
      const sockets: FakeSocket[] = [];
      class FakeSocket {
        onopen: ((ev: unknown) => void) | null = null;
        onmessage: ((ev: { data: unknown }) => void) | null = null;
        onerror: ((ev: unknown) => void) | null = null;
        onclose: ((ev: unknown) => void) | null = null;
        constructor(public url: string) { sockets.push(this); }
        send(): void {}
        close(): void {}
      }
      const controller = createRealtimeController({
        onChanged,
        push: {
          WebSocketImpl: FakeSocket as never,
          backoff: [10],
          checkSession: () => Promise.resolve('ok'),
          heartbeatMs: 60000,
        },
      });
      controller.reconciler.seed('acc', { Email: '1' });
      controller.start();
      sockets[0]!.onopen?.({});
      expect(onChanged).not.toHaveBeenCalled();

      sockets[0]!.onclose?.({});
      vi.advanceTimersByTime(20);
      sockets[1]!.onopen?.({});
      expect(onChanged).toHaveBeenCalledTimes(1);
      expect(onChanged).toHaveBeenCalledWith('acc', ['Email', 'Mailbox', 'EmailSubmission', 'Thread']);
      controller.stop();
    } finally {
      vi.useRealTimers();
    }
  });
});
