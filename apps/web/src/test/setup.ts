import '@testing-library/jest-dom/vitest';
import { afterEach } from 'vitest';
import { configure } from '@solidjs/testing-library';
import { setupI18nForTests } from './i18n.ts';
import { resetToasts } from '../components/toast/toast.ts';
import { forgetSession } from '../state/sessionRestore.ts';
import { WORKSPACE_STORAGE_KEY } from '../realtime/workspaceStorage.ts';
import { resetRateState } from '../api/rateAware.ts';

// Seed the i18n registry with the `en` catalogs so `t(id)` resolves to English
// synchronously — keeps literal-text assertions green as strings get wrapped.
setupI18nForTests();

// The toast store (components/toast) is module state: a toast one test raised
// must not be on screen in the next.
afterEach(() => resetToasts());

// t109: what a browser tab keeps to restore after a reload (sessionStorage and
// the address-bar target) is per tab; each test is a fresh tab.
afterEach(() => {
  forgetSession();
  try {
    for (const key of Object.keys(sessionStorage)) if (key.startsWith(`${WORKSPACE_STORAGE_KEY}:`)) sessionStorage.removeItem(key);
  } catch { /* no storage */ }
});

// The request pacing (api/rateAware) answers identical in-flight GETs once and
// keeps backoff state at module level: a request one test left pending, or a
// Retry-After window it opened, must not reach the next test.
afterEach(() => resetRateState());

// A parallel run of ~760 files slows jsdom rendering well past the 1 s default of
// findBy*/waitFor; a test that is right fails only when the machine is busy.
configure({ asyncUtilTimeout: 5000 });

// No test reaches a real socket. jsdom's own WebSocket dials the test origin
// (localhost:3000); when something on the host listens there and never answers
// the upgrade, every request apiFetch offers to the WebSocket action transport
// waits out its 5 s handshake timer before falling back to HTTP (5 s per test,
// only on such a host). This stand-in refuses the connection the way a server
// without the action endpoint does. A test that needs a socket stubs its own
// with vi.stubGlobal('WebSocket', ...), which replaces this one.
class RefusedWebSocket extends EventTarget {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSING = 2;
  readonly CLOSED = 3;
  readyState = 0;
  binaryType: BinaryType = 'blob';
  bufferedAmount = 0;
  extensions = '';
  protocol = '';
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  constructor(readonly url: string | URL) {
    super();
    setTimeout(() => {
      if (this.readyState === 3) return;
      this.readyState = 3;
      const error = new Event('error');
      this.onerror?.(error);
      this.dispatchEvent(error);
      const close = new CloseEvent('close', { code: 1006, wasClean: false });
      this.onclose?.(close);
      this.dispatchEvent(close);
    }, 0);
  }
  send(): void {
    throw new DOMException('WebSocket is not open', 'InvalidStateError');
  }
  close(): void {
    this.readyState = 3;
  }
}
globalThis.WebSocket = RefusedWebSocket as unknown as typeof WebSocket;
