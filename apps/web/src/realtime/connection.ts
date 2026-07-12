// Connection-status model (plan §3 e6 connection-status toasts).
//
// Reactive state derived from the push client's lifecycle plus two out-of-band
// signals the socket layer can't see on its own: `offline` (the fetch layer's
// network events) and `auth-expired` (a 401 from a `*/changes` refetch). The
// `ConnectionToast` component renders from `state`; keeping a single reactive
// value is what dedupes the surface — no repeated toasts for the same status.

import { createSignal, type Accessor } from 'solid-js';
import type { PushStatus } from './pushClient.ts';
import type { PushTransport } from '../contracts/push.ts';

export type ConnectionState =
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
  /** The browser went offline (fetch network-down). Sticky until online. */
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
      return 'offline';
  }
}

export function createConnection(): ConnectionModel {
  const [state, setState] = createSignal<ConnectionState>('offline');
  const [transport, setTransport] = createSignal<PushTransport>('offline');
  // Auth expiry outranks socket lifecycle: a reconnecting socket must not hide a
  // dead session. It clears only when the socket reports a healthy 'open'.
  let authExpired = false;

  function report(status: PushStatus, t: PushTransport): void {
    setTransport(t);
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
      setTransport('offline');
      if (!authExpired) setState('offline');
    },
    setAuthExpired(): void {
      authExpired = true;
      setState('auth-expired');
    },
  };
}
