import { describe, expect, it, vi } from 'vitest';
import { wireServiceWorkerWake } from './realtime.ts';

function wakeMessage(type: string): MessageEvent {
  return new MessageEvent('message', { data: { type } });
}

describe('wireServiceWorkerWake', () => {
  it('reconnects on a mw-push-wake message', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    wireServiceWorkerWake({ reconnect }, target);

    target.dispatchEvent(wakeMessage('mw-push-wake'));
    expect(reconnect).toHaveBeenCalledTimes(1);
  });

  it('ignores unrelated service-worker messages', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    wireServiceWorkerWake({ reconnect }, target);

    target.dispatchEvent(wakeMessage('other'));
    target.dispatchEvent(new MessageEvent('message'));
    expect(reconnect).not.toHaveBeenCalled();
  });

  it('is inert (no throw) when no service worker is available', () => {
    const reconnect = vi.fn();
    const cleanup = wireServiceWorkerWake({ reconnect }, undefined);
    expect(reconnect).not.toHaveBeenCalled();
    expect(() => cleanup()).not.toThrow();
  });

  it('cleanup removes the listener', () => {
    const target = new EventTarget();
    const reconnect = vi.fn();
    const cleanup = wireServiceWorkerWake({ reconnect }, target);
    cleanup();

    target.dispatchEvent(wakeMessage('mw-push-wake'));
    expect(reconnect).not.toHaveBeenCalled();
  });
});
