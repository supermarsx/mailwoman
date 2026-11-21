// Service-worker registration: in a browser the worker is registered under the
// deploy prefix; in a Tauri shell none is, and one an earlier build registered is
// removed (t31-e7: a worker's fetches leave the WebView, so on Windows the shell's
// origin is answered by whatever listens on loopback port 80).

import { afterEach, describe, expect, it, vi } from 'vitest';
import { registerServiceWorker,requestBackgroundReplay,backgroundReplayState } from './register.ts';

interface G {
  __TAURI_INTERNALS__?: unknown;
}

function fakeServiceWorker(): { register: ReturnType<typeof vi.fn>; unregister: ReturnType<typeof vi.fn> } {
  const unregister = vi.fn(async () => true);
  const register = vi.fn(async () => ({}));
  Object.defineProperty(navigator, 'serviceWorker', {
    configurable: true,
    value: { register, getRegistrations: async () => [{ unregister }, { unregister }] },
  });
  return { register, unregister };
}

afterEach(async () => {
  delete (globalThis as G).__TAURI_INTERNALS__;
  Reflect.deleteProperty(navigator, 'serviceWorker');
  await requestBackgroundReplay();
  vi.useRealTimers();
});

describe('registerServiceWorker', () => {
  it('registers the worker in a browser', async () => {
    const sw = fakeServiceWorker();
    await registerServiceWorker();
    expect(sw.register).toHaveBeenCalledWith('/sw.js', { type: 'classic', scope: '/' });
    expect(sw.unregister).not.toHaveBeenCalled();
  });

  it('registers nothing in a Tauri shell and removes earlier registrations', async () => {
    (globalThis as G).__TAURI_INTERNALS__ = {};
    const sw = fakeServiceWorker();
    await registerServiceWorker();
    expect(sw.register).not.toHaveBeenCalled();
    expect(sw.unregister).toHaveBeenCalledTimes(2);
  });
});

describe('foreground replay fallback',()=>{
  it('replays through the active worker after the actual browser controllerchange lifecycle event',async()=>{
    const postMessage=vi.fn(),container=new EventTarget();
    Object.assign(container,{register:vi.fn(async()=>({})),getRegistration:async()=>({active:{state:'activated',postMessage}})});
    Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:container});
    await registerServiceWorker();
    container.dispatchEvent(new Event('controllerchange'));
    await vi.waitFor(()=>expect(postMessage).toHaveBeenCalledExactlyOnceWith({type:'mw-outbox-replay'}));
    expect(backgroundReplayState()).toBe('unavailable');
  });
  it('wakes the active worker without claiming closed-window replay when Background Sync is absent',async()=>{
    const postMessage=vi.fn();Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>({active:{state:'activated',postMessage}})}});
    expect(await requestBackgroundReplay()).toBe(false);expect(backgroundReplayState()).toBe('unavailable');expect(postMessage).toHaveBeenCalledExactlyOnceWith({type:'mw-outbox-replay'});
  });
  it('uses the same foreground fallback when Background Sync registration is refused',async()=>{
    const postMessage=vi.fn(),register=vi.fn(async()=>{throw new Error('Browser policy refused');});Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>({active:{state:'activated',postMessage},sync:{register}})}});
    expect(await requestBackgroundReplay()).toBe(false);expect(register).toHaveBeenCalledExactlyOnceWith('mw-outbox');expect(postMessage).toHaveBeenCalledExactlyOnceWith({type:'mw-outbox-replay'});
  });
  it('keeps Background Sync registration as the closed-window capability',async()=>{
    const postMessage=vi.fn(),register=vi.fn(async()=>undefined);Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>({active:{state:'activated',postMessage},sync:{register}})}});
    expect(await requestBackgroundReplay()).toBe(true);expect(backgroundReplayState()).toBe('available');expect(register).toHaveBeenCalledExactlyOnceWith('mw-outbox');expect(postMessage).not.toHaveBeenCalled();
  });
});

it('coalesces foreground retries and stops when Background Sync becomes available',async()=>{
 vi.useFakeTimers();const postMessage=vi.fn(),register=vi.fn(async()=>undefined);
 const registration:{active:{state:string;postMessage:typeof postMessage};sync?:{register:typeof register}}={active:{state:'activated',postMessage}};
 Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>registration}});
 await requestBackgroundReplay();await requestBackgroundReplay();expect(vi.getTimerCount()).toBe(1);
 await vi.advanceTimersByTimeAsync(60000);expect(postMessage).toHaveBeenCalledTimes(3);expect(vi.getTimerCount()).toBe(1);expect(backgroundReplayState()).toBe('unavailable');
 registration.sync={register};await vi.advanceTimersByTimeAsync(60000);expect(register).toHaveBeenCalledWith('mw-outbox');expect(vi.getTimerCount()).toBe(0);expect(backgroundReplayState()).toBe('available');
});
it('stops retries when the active registration disappears',async()=>{
 vi.useFakeTimers();let active=true;
 Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>active?{active:{state:'activated',postMessage:vi.fn()}}:undefined}});
 await requestBackgroundReplay();expect(vi.getTimerCount()).toBe(1);active=false;await vi.advanceTimersByTimeAsync(60000);expect(vi.getTimerCount()).toBe(0);
});
