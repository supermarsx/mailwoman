// Connection-status model (plan §3 e6 connection-status toasts).
//
// Reactive state derived from the push client's lifecycle plus two out-of-band
// signals the socket layer can't see on its own: `offline` (the fetch layer's
// network events) and `auth-expired` (a 401 from a `*/changes` refetch). The
// `ConnectionToast` component renders from `state`; keeping a single reactive
// value is what dedupes the surface — no repeated toasts for the same status.
//
// `idle` is "no push transport is wanted": before the first `start()` and after
// a `stop()`, which is every signed-out screen and the forced-password-change
// hold. It is not a connectivity claim, so it must not read as `offline` — that
// is what put "You are offline" on the login screen of an online browser.

import { createSignal, type Accessor } from 'solid-js';
import type { PushStatus } from './pushClient.ts';
import type { PushTransport } from '../contracts/push.ts';

export type ConnectionState =
  | 'idle'
  | 'online'
  | 'connecting'
  | 'degraded'
  | 'offline'
  | 'auth-expired';

export interface ConnectionModel {
  state: Accessor<ConnectionState>;
  transport: Accessor<PushTransport>;
  /** Fed by the push client on every lifecycle change. */
  report(status: PushStatus, transport: PushTransport): void;
  /** A request failed at the network layer. Sticky until the push client next
   *  reports. Ignored while idle: no push client is running to clear it. */
  setOffline(): void;
  /** A refetch returned 401. Sticky until a successful reconnect clears it. */
  setAuthExpired(): void;
}

function mapStatus(status: PushStatus): ConnectionState {
  switch (status) {
    case 'open':
      return 'online';
    case 'connecting':
    case 'reconnecting':
      return 'connecting';
    case 'degraded':
      return 'degraded';
    case 'closed':
      return 'idle';
  }
}

export function createConnection(): ConnectionModel {
  const [state, setState] = createSignal<ConnectionState>('idle');
  const [transport, setTransport] = createSignal<PushTransport>('offline');
  // Auth expiry outranks socket lifecycle: a reconnecting socket must not hide a
  // dead session. It clears only when the socket reports a healthy 'open'.
  let authExpired = false;
  // True between the push client's first report after `connect()` and its
  // 'closed'. Only then is there a connection that can be lost.
  let active = false;

  function report(status: PushStatus, t: PushTransport): void {
    setTransport(t);
    active = status !== 'closed';
    if (status === 'open') authExpired = false;
    if (authExpired) {
      setState('auth-expired');
      return;
    }
    setState(mapStatus(status));
  }

  return {
    state,
    transport,
    report,
    setOffline(): void {
      if (!active) return;
      setTransport('offline');
      if (!authExpired) setState('offline');
    },
    setAuthExpired(): void {
      authExpired = true;
      setState('auth-expired');
    },
  };
}
