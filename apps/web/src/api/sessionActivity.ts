import { apiFetch } from './http.ts';
import { isConsolePage } from './adminPath.ts';
import { withBase } from './basePath.ts';
import { observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent, type PreferenceAccount } from './accountPreferenceLifecycle.ts';

const INTERVAL_MS = 30_000;
/** No timer: only another genuine foreground interaction can send another renewal. */
export function createSessionActivity(options: {
  now?: () => number;
  foreground?: () => boolean;
  send?: (signal: AbortSignal) => Promise<Response>;
} = {}): { activity: (trusted: boolean) => void; dispose: () => void } {
  const now = options.now ?? Date.now;
  const foreground = options.foreground ?? (() => !isConsolePage() && document.visibilityState === 'visible' && document.hasFocus());
  const send = options.send ?? ((signal: AbortSignal) => apiFetch(withBase('/api/account/session/activity'), { method: 'POST', signal }));
  let lastSent = -Infinity;
  let pending: AbortController | undefined;
  let blocked: PreferenceAccount | null = null;
  const stop = observePreferenceAccount(() => { pending?.abort(); pending = undefined; lastSent = -Infinity; blocked = null; });
  function activity(trusted: boolean): void {
    const account = preferenceAccount();
    if (!trusted || !foreground() || !account || pending || (blocked && preferenceAccountIsCurrent(blocked)) || now() - lastSent < INTERVAL_MS) return;
    lastSent = now();
    const controller = new AbortController(); pending = controller;
    void send(controller.signal).then((response) => {
      if (preferenceAccountIsCurrent(account) && response.status === 401) blocked = account;
    }).catch(() => { /* Another real interaction may retry after the throttle. */ }).finally(() => {
      if (pending === controller) pending = undefined;
    });
  }
  return { activity, dispose: () => { stop(); pending?.abort(); pending = undefined; } };
}

let started = false;
export function startMailboxSessionActivity(): void {
  if (started || typeof document === 'undefined') return;
  started = true;
  const tracker = createSessionActivity();
  const interaction = (event: Event): void => tracker.activity(event.isTrusted);
  // Plain scroll/visibility/focus events are intentionally excluded: scripted
  // scrolling and background refresh must not keep an unattended session alive.
  for (const name of ['pointerdown', 'keydown', 'wheel', 'touchstart']) {
    document.addEventListener(name, interaction, { passive: true, capture: true });
  }
}
