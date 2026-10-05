// Connection-status model (plan §3 e6 connection-status toasts).
//
// Reactive state derived from the push client's lifecycle plus two out-of-band
// signals the socket layer can't see on its own: `offline` (the fetch layer's
// network events) and `auth-expired` (a 401 on an authenticated request). The
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
  /** A request failed at the network layer. Held until `setReachable()` or the
   *  push client's next report. Ignored while idle: there is no session
   *  connection to have lost. */
  setOffline(): void;
  /** A request reached the server again. Undoes `setOffline()` by returning to
   *  what the push client last reported; changes nothing otherwise, so it
   *  cannot hide a socket that is itself reconnecting or degraded. */
  setReachable(): void;
  /** An authenticated request was answered 401: the session has ended. Held
   *  until a healthy 'open' or the transport's close (sign-out). Ignored while
   *  idle: a 401 with no session in use is the signed-out answer. */
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
  // dead session. It clears when the socket reports a healthy 'open', and when
  // the transport is closed: sign-out ends the session the expiry was about.
  let authExpired = false;
  // True between the push client's first report after `connect()` and its
  // 'closed'. Only then is there a connection that can be lost.
  let active = false;
  // The push client's last report, restored when the network comes back.
  let last: { status: PushStatus; transport: PushTransport } | null = null;
  // True while `setOffline()` is what the state shows.
  let netDown = false;

  function report(status: PushStatus, t: PushTransport): void {
    setTransport(t);
    active = status !== 'closed';
    last = { status, transport: t };
    netDown = false;
    if (status === 'open' || status === 'closed') authExpired = false;
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
      netDown = true;
      setTransport('offline');
      if (!authExpired) setState('offline');
    },
    setReachable(): void {
      if (!netDown || last === null) return;
      netDown = false;
      setTransport(last.transport);
      if (!authExpired) setState(mapStatus(last.status));
    },
    setAuthExpired(): void {
      if (!active) return;
      authExpired = true;
      setState('auth-expired');
    },
  };
}
