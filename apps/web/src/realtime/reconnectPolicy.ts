// Reconnect timing shared by every realtime socket (t35-e4). Pure functions of
// the failure count, the close code and an injectable random source, so the
// push stream and the action sockets can apply one policy and tests can pin it.

import {
  RECONNECT_BASE_MS,
  RECONNECT_CAP_MS,
  RECONNECT_MAX_FAILURES,
  RESTART_SPREAD_MS,
} from '../contracts/push.ts';

/** How a socket ended, as far as the retry timing is concerned. */
export type CloseKind =
  /** Server closed with 1008: the session is gone. */
  | 'ended'
  /** 1012: the server is restarting. */
  | 'restart'
  /** 1013: the server asked clients to try again later. */
  | 'busy'
  /** Anything else: 1006, a failed handshake, a silent socket. */
  | 'drop';

export function closeKind(code: unknown): CloseKind {
  if (code === 1008) return 'ended';
  if (code === 1012) return 'restart';
  if (code === 1013) return 'busy';
  return 'drop';
}

export interface BackoffConfig {
  baseMs?: number;
  capMs?: number;
  restartSpreadMs?: number;
}

/**
 * Delay before the next attempt after `failures` consecutive failures (>= 1).
 * - drop: uniform in [0, min(cap, base * 2^(failures-1))]
 * - restart: the same, but the ceiling is at least restartSpreadMs
 * - busy: uniform in [restartSpreadMs, ceiling], ceiling at least twice that spread (never above cap)
 */
export function reconnectDelay(
  kind: CloseKind,
  failures: number,
  random: () => number = Math.random,
  config: BackoffConfig = {},
): number {
  const base = config.baseMs ?? RECONNECT_BASE_MS;
  const cap = config.capMs ?? RECONNECT_CAP_MS;
  const spread = config.restartSpreadMs ?? RESTART_SPREAD_MS;
  const n = Math.min(Math.max(Math.floor(failures), 1), RECONNECT_MAX_FAILURES);
  const ceiling = Math.min(cap, base * 2 ** (n - 1));
  const r = Math.min(Math.max(random(), 0), 0.999999);
  if (kind === 'restart') return Math.floor(r * Math.max(spread, ceiling));
  if (kind === 'busy') {
    const high = Math.min(cap, Math.max(2 * spread, ceiling));
    const low = Math.min(spread, high);
    return Math.floor(low + r * (high - low));
  }
  return Math.floor(r * ceiling);
}

/** `value` spread by plus or minus `fraction`, uniformly. */
export function jittered(value: number, fraction: number, random: () => number = Math.random): number {
  const r = Math.min(Math.max(random(), 0), 0.999999);
  return Math.round(value * (1 - fraction + 2 * fraction * r));
}
