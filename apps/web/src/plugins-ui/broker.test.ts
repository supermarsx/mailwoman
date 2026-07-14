import { describe, it, expect, vi } from 'vitest';
import { classifyMessage, handleGuestMessage, attachBroker, type BrokerWiring } from './broker';
import type { RpcRequest, RpcResponse, UiPluginGrant } from './types';

const frameWindow = {} as unknown as Window;
const grants: readonly UiPluginGrant[] = [
  { capability: 'net:host-allowlist', params: { hosts: ['api.example.com'] } },
];

function evt(data: unknown, source: unknown = frameWindow, origin = 'null') {
  return { data, source: source as MessageEventLike['source'], origin };
}
type MessageEventLike = Pick<MessageEvent, 'source' | 'origin' | 'data'>;

const fetchReq: RpcRequest = { v: 1, id: 'r1', cap: 'net:host-allowlist', method: 'fetch', args: ['https://api.example.com/x'] };

describe('classifyMessage (deny-by-default, pure — the e15 escape-gate hook)', () => {
  it('forwards a trusted, granted, allow-listed call', () => {
    const d = classifyMessage(evt(fetchReq), grants, frameWindow);
    expect(d).toEqual({ kind: 'forward', request: fetchReq });
  });

  it('IGNORES a foreign-origin / spoofed-source message (escape attempt blocked)', () => {
    const attacker = {} as unknown as Window;
    const d = classifyMessage(evt(fetchReq, attacker), grants, frameWindow);
    expect(d).toEqual({ kind: 'ignore', reason: 'foreign-origin' });
  });

  it('IGNORES a concrete-origin message even from the right window', () => {
    const d = classifyMessage(evt(fetchReq, frameWindow, 'https://host.example'), grants, frameWindow);
    expect(d).toEqual({ kind: 'ignore', reason: 'foreign-origin' });
  });

  it('IGNORES a malformed payload (no id to answer)', () => {
    const d = classifyMessage(evt({ nope: true }), grants, frameWindow);
    expect(d).toEqual({ kind: 'ignore', reason: 'malformed' });
  });

  it('REJECTS an ungranted capability with capability-denied', () => {
    const req = { v: 1, id: 'r2', cap: 'store:kv-scoped', method: 'get', args: ['k'] };
    const d = classifyMessage(evt(req), grants, frameWindow);
    expect(d.kind).toBe('reject');
    if (d.kind === 'reject') {
      expect('err' in d.response && d.response.err.code).toBe('capability-denied');
    }
  });

  it('REJECTS a non-allow-listed method with method-denied', () => {
    const req = { v: 1, id: 'r3', cap: 'net:host-allowlist', method: 'delete', args: [] };
    const d = classifyMessage(evt(req), grants, frameWindow);
    expect(d.kind).toBe('reject');
    if (d.kind === 'reject') {
      expect('err' in d.response && d.response.err.code).toBe('method-denied');
    }
  });
});

describe('handleGuestMessage (forward + relay)', () => {
  function wiring(over: Partial<BrokerWiring> = {}): { w: BrokerWiring; posts: RpcResponse[]; rpc: ReturnType<typeof vi.fn> } {
    const posts: RpcResponse[] = [];
    const rpc = vi.fn(async (_id: string, req: RpcRequest): Promise<RpcResponse> => ({ v: 1, id: req.id, ok: { status: 200 } }));
    const w: BrokerWiring = {
      pluginId: 'snooze',
      grants,
      frameWindow,
      post: (r) => posts.push(r),
      rpc,
      ...over,
    };
    return { w, posts, rpc };
  }

  it('forwards an allow-listed call to the server broker and relays ok', async () => {
    const { w, posts, rpc } = wiring();
    await handleGuestMessage(evt(fetchReq), w);
    expect(rpc).toHaveBeenCalledWith('snooze', fetchReq);
    expect(posts).toEqual([{ v: 1, id: 'r1', ok: { status: 200 } }]);
  });

  it('rejects method-denied LOCALLY — never touches the network', async () => {
    const { w, posts, rpc } = wiring();
    const req = { v: 1, id: 'm1', cap: 'net:host-allowlist', method: 'delete', args: [] };
    await handleGuestMessage(evt(req), w);
    expect(rpc).not.toHaveBeenCalled();
    expect(posts).toHaveLength(1);
    expect('err' in posts[0]! && posts[0]!.err.code).toBe('method-denied');
  });

  it('drops a foreign-origin message with no reply', async () => {
    const { w, posts, rpc } = wiring();
    const attacker = {} as unknown as Window;
    await handleGuestMessage(evt(fetchReq, attacker), w);
    expect(rpc).not.toHaveBeenCalled();
    expect(posts).toHaveLength(0);
  });

  it('relays an internal error when the server rpc throws', async () => {
    const rpc = vi.fn(async () => {
      throw new Error('down');
    });
    const { w, posts } = wiring({ rpc });
    await handleGuestMessage(evt(fetchReq), w);
    expect('err' in posts[0]! && posts[0]!.err.code).toBe('internal');
  });
});

describe('attachBroker (listener lifecycle)', () => {
  it('adds + removes a message listener on the target window', () => {
    const add = vi.fn();
    const remove = vi.fn();
    const target = { addEventListener: add, removeEventListener: remove } as unknown as Window;
    const disconnect = attachBroker(target, { pluginId: 'p', grants, frameWindow });
    expect(add).toHaveBeenCalledWith('message', expect.any(Function));
    disconnect();
    expect(remove).toHaveBeenCalledWith('message', expect.any(Function));
    // Same listener reference added + removed.
    expect(add.mock.calls[0]![1]).toBe(remove.mock.calls[0]![1]);
  });
});
