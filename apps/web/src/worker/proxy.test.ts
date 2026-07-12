import { describe, it, expect, vi } from 'vitest';
import {
  channelTransport,
  createStoreProxy,
  sharedWorkerAvailable,
  workerTransport,
  type SharedWorkerLike,
} from './proxy.ts';
import { createWorkerCore } from './workerCore.ts';
import type { PortLike } from './protocol.ts';

/** A linked pair of ports: each `postMessage` delivers to the other's handler. */
function linkedPorts(): [PortLike, PortLike] {
  const pair: { a: PortLike; b: PortLike } = {
    a: {
      postMessage: (d: unknown) => queueMicrotask(() => pair.b.onmessage?.({ data: d })),
      onmessage: null,
    },
    b: {
      postMessage: (d: unknown) => queueMicrotask(() => pair.a.onmessage?.({ data: d })),
      onmessage: null,
    },
  };
  return [pair.a, pair.b];
}

const flush = (): Promise<void> => new Promise((r) => setTimeout(r, 0));

describe('workerTransport ↔ workerCore', () => {
  it('round-trips a request to the worker handler', async () => {
    const [clientPort, serverPort] = linkedPorts();
    const core = createWorkerCore({ echo: (params) => params });
    core.connect(serverPort);
    const tx = workerTransport(clientPort);
    await expect(tx.request('echo', { x: 1 })).resolves.toEqual({ x: 1 });
  });

  it('rejects an unknown method with a structured error', async () => {
    const [clientPort, serverPort] = linkedPorts();
    createWorkerCore({}).connect(serverPort);
    const tx = workerTransport(clientPort);
    await expect(tx.request('nope')).rejects.toMatchObject({ message: expect.stringContaining('nope') });
  });

  it('surfaces a handler-thrown error to the caller', async () => {
    const [clientPort, serverPort] = linkedPorts();
    createWorkerCore({
      boom: () => {
        throw new Error('kaboom');
      },
    }).connect(serverPort);
    const tx = workerTransport(clientPort);
    await expect(tx.request('boom')).rejects.toMatchObject({ message: 'kaboom' });
  });

  it('delivers worker broadcasts to subscribers', async () => {
    const [clientPort, serverPort] = linkedPorts();
    const core = createWorkerCore({
      bump: (_p, broadcast) => {
        broadcast({ n: 1 });
        return 'ok';
      },
    });
    core.connect(serverPort);
    const tx = workerTransport(clientPort);
    const deltas: unknown[] = [];
    tx.onBroadcast((d) => deltas.push(d));
    await tx.request('bump');
    await flush();
    expect(deltas).toEqual([{ n: 1 }]);
  });

  it('fans a core broadcast out to every connected port', async () => {
    const core = createWorkerCore({});
    const [c1, s1] = linkedPorts();
    const [c2, s2] = linkedPorts();
    core.connect(s1);
    core.connect(s2);
    expect(core.portCount()).toBe(2);
    const t1 = workerTransport(c1);
    const t2 = workerTransport(c2);
    const seen1: unknown[] = [];
    const seen2: unknown[] = [];
    t1.onBroadcast((d) => seen1.push(d));
    t2.onBroadcast((d) => seen2.push(d));
    core.broadcast({ hello: true });
    await flush();
    expect(seen1).toEqual([{ hello: true }]);
    expect(seen2).toEqual([{ hello: true }]);
  });
});

describe('channelTransport (BroadcastChannel fallback)', () => {
  it('runs requests against the tab-local store', async () => {
    const [port] = linkedPorts();
    const local = vi.fn(async (method: string) => `local:${method}`);
    const tx = channelTransport(port, local);
    expect(tx.mode).toBe('channel');
    await expect(tx.request('flag', { id: 'm1' })).resolves.toBe('local:flag');
    expect(local).toHaveBeenCalledWith('flag', { id: 'm1' });
  });

  it('gossips published deltas to peer tabs but never echoes to the sender', async () => {
    const [portA, portB] = linkedPorts();
    const txA = channelTransport(portA, async () => undefined);
    const txB = channelTransport(portB, async () => undefined);
    const aSeen: unknown[] = [];
    const bSeen: unknown[] = [];
    txA.onBroadcast((d) => aSeen.push(d));
    txB.onBroadcast((d) => bSeen.push(d));
    txA.publish({ moved: 'm1' });
    await flush();
    expect(bSeen).toEqual([{ moved: 'm1' }]);
    expect(aSeen).toEqual([]);
  });
});

describe('createStoreProxy — transport selection', () => {
  function fakeSharedWorker(port: PortLike): new (url: string | URL, name?: string) => SharedWorkerLike {
    return class implements SharedWorkerLike {
      port: PortLike;
      constructor() {
        this.port = port;
      }
    };
  }

  it('prefers a SharedWorker when one is available', () => {
    const [clientPort, serverPort] = linkedPorts();
    createWorkerCore({}).connect(serverPort);
    const local = vi.fn();
    const tx = createStoreProxy({
      SharedWorkerImpl: fakeSharedWorker(clientPort),
      localRequest: local,
    });
    expect(tx.mode).toBe('worker');
    expect(local).not.toHaveBeenCalled();
  });

  it('falls back to the BroadcastChannel when no SharedWorker exists', () => {
    const [port] = linkedPorts();
    const tx = createStoreProxy({ channelPort: port, localRequest: async () => 'x' });
    expect(tx.mode).toBe('channel');
  });

  it('still works as an isolated tab when neither transport exists', async () => {
    const local = vi.fn(async () => 'solo');
    const tx = createStoreProxy({ channelPort: null, localRequest: local });
    expect(tx.mode).toBe('channel');
    await expect(tx.request('flag')).resolves.toBe('solo');
  });

  it('sharedWorkerAvailable reflects the injected/global ctor', () => {
    expect(sharedWorkerAvailable(fakeSharedWorker(linkedPorts()[0]))).toBe(true);
    // jsdom has no SharedWorker global.
    expect(sharedWorkerAvailable()).toBe(false);
  });
});
