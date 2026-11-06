// Waiting out a server restart without signing anyone out.
//
// While the server restarts (a deploy, a crash, a container recreate) the proxy in
// front of it answers 502/503/504 and a direct connection is refused. None of
// those say anything about the session: only a 401 does. So the session check at
// page load retries on exponential backoff with jitter and shows a quiet
// "Reconnecting…" line, instead of the sign-in form or a boot-error card.

import { createSignal, type Accessor } from 'solid-js';
import { ApiError, NetworkError } from '../api/client.ts';
import { backoffDelay } from '../api/recovery.ts';

const [reconnecting, setReconnecting] = createSignal(false);

/** True while the page load is waiting for an unreachable server to come back. */
export const bootReconnecting: Accessor<boolean> = reconnecting;

/** The server was not reached, or a proxy in front of it could not reach it. */
export function isServerUnreachable(err: unknown): boolean {
  if (err instanceof NetworkError) return true;
  return err instanceof ApiError && (err.status === 502 || err.status === 503 || err.status === 504);
}

export interface BootRetryOptions {
  /** Most sends in all. The default (14) waits roughly a minute in total. */
  attempts?: number;
  baseDelayMs?: number;
  maxDelayMs?: number;
  /** Replaces the wait (tests). */
  sleep?: (ms: number) => Promise<void>;
}

const wait = (ms: number): Promise<void> => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Run `attempt`, repeating it while the server is unreachable. Any other outcome
 * (an answer, a 401, a refusal) is returned or thrown at once; after the last
 * attempt the unreachable error is thrown so the boot-error card can offer Retry.
 */
export async function retryWhileUnreachable<T>(attempt: () => Promise<T>, opts: BootRetryOptions = {}): Promise<T> {
  const attempts = opts.attempts ?? 14;
  const sleep = opts.sleep ?? wait;
  try {
    for (let n = 1; ; n += 1) {
      try {
        return await attempt();
      } catch (err) {
        if (!isServerUnreachable(err) || n >= attempts) throw err;
        setReconnecting(true);
        await sleep(backoffDelay(n, opts.baseDelayMs ?? 500, opts.maxDelayMs ?? 5000));
      }
    }
  } finally {
    setReconnecting(false);
  }
}
