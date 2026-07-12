// SharedWorker bootstrap entry (plan §2.6). Loaded as its own bundle via
// `new SharedWorker(new URL('./sharedWorker.ts', import.meta.url), { type: 'module' })`
// (see proxy.ts). It adapts each connecting `MessagePort` to the transport-
// agnostic `PortLike` and hands it to the store core. The real store handlers
// (JMAP client-backed) are registered at integration (Batch C); until then the
// core answers a liveness `ping` so the wiring is verifiable end to end.
//
// The SharedWorker global scope (`onconnect`) is not in the app's DOM lib, so it
// is typed structurally here rather than pulling in the WebWorker lib.

import { createWorkerCore } from './workerCore.ts';
import type { PortLike } from './protocol.ts';

interface ConnectEvent {
  readonly ports: readonly MessagePort[];
}
interface SharedWorkerScope {
  onconnect: ((ev: ConnectEvent) => void) | null;
}

/** Adapt a DOM `MessagePort` to the structural `PortLike` the core speaks. */
function adaptPort(mp: MessagePort): PortLike {
  const port: PortLike = {
    postMessage: (data: unknown): void => mp.postMessage(data),
    onmessage: null,
    start: (): void => mp.start(),
    close: (): void => mp.close(),
  };
  mp.onmessage = (ev: MessageEvent): void => port.onmessage?.({ data: ev.data });
  return port;
}

const core = createWorkerCore({
  ping: () => 'pong',
  // Integration (Batch C) registers the real store methods here.
});

const scope = self as unknown as SharedWorkerScope;
scope.onconnect = (ev: ConnectEvent): void => {
  for (const mp of ev.ports) core.connect(adaptPort(mp));
};
