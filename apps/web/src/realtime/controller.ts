// Realtime controller (plan §3 e6): the single object the UI reads for push
// connection status and sub-tabs. It owns the push client, the connection-state
// model, the change reconciler, and the sub-tab strip, and wires the push
// client's lifecycle into the connection model. Components consume it via
// `RealtimeContext` (context.ts); the realtime store slice constructs the app's
// singleton and starts/stops it around the session.

import { createPushClient, type PushClientImpl, type PushClientOptions } from './pushClient.ts';
import { createConnection, type ConnectionModel } from './connection.ts';
import { createSubTabs, type SubTabsModel, type SubTabsOptions } from './subTabs.ts';
import {
  createChangeReconciler,
  type ChangeReconciler,
  type ReconcileHandler,
} from './changes.ts';
import type { StateChange } from '../contracts/push.ts';

export interface RealtimeController {
  connection: ConnectionModel;
  subTabs: SubTabsModel;
  reconciler: ChangeReconciler;
  /** Open the push transport and begin reacting to `StateChange`. */
  start(): void;
  /** Close the transport (logout / teardown). */
  stop(): void;
  /** Force a reconnect from the top of the transport ladder (toast action). */
  reconnect(): void;
  /** Subscribe to decoded state changes (post-reconciliation is the slice's job). */
  onStateChange(handler: (c: StateChange) => void): () => void;
}

export interface RealtimeControllerOptions {
  push?: PushClientOptions;
  subTabs?: SubTabsOptions;
  /** Provide a pre-built push client (tests inject a fake). */
  pushClient?: PushClientImpl;
  /** Invoked with the datatypes that moved for an account after each push. */
  onChanged?: ReconcileHandler;
}

export function createRealtimeController(
  opts: RealtimeControllerOptions = {},
): RealtimeController {
  const connection = createConnection();
  const subTabs = createSubTabs(opts.subTabs);
  const reconciler = createChangeReconciler(opts.onChanged ?? (() => undefined));

  const push =
    opts.pushClient ??
    createPushClient({
      ...opts.push,
      onStatus: (status, transport) => {
        connection.report(status, transport);
        opts.push?.onStatus?.(status, transport);
      },
    });

  // Every pushed StateChange runs through the reconciler (which fires
  // `onChanged` for the moved datatypes → `*/changes` refetch at integration).
  push.onStateChange((c) => reconciler.apply(c));

  return {
    connection,
    subTabs,
    reconciler,
    start(): void {
      push.connect();
    },
    stop(): void {
      push.close();
      reconciler.reset();
    },
    reconnect(): void {
      push.reconnect();
    },
    onStateChange(handler): () => void {
      return push.onStateChange(handler);
    },
  };
}
