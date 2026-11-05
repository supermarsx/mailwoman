// "A new version is available" notice (user request: tell signed-in users when a
// new webmail has been deployed).
//
// The HTML shell can be older than the assets and server it talks to: a tab left
// open across a deploy keeps running the old code. The server reports the build it
// serves at `GET /api/version` (session required). The first answer after sign-in
// is this page's own build (a page is loaded from the build the server serves at
// that moment); a later, different answer means a deploy happened since.
//
// Checks run when the window regains focus or turns visible, when the network
// returns, and on a slow jittered poll, only while signed in and only when the
// `ui.notifyNewVersion` setting is on. A mismatch shows ONE toast (same id, so a
// second check never stacks one). Closing the toast is "Later": the notice for
// that build is silenced for an hour, remembered per tab in sessionStorage.

export const SNOOZE_MS = 60 * 60_000;
export const POLL_MS = 10 * 60_000;
/** Focus and visibility events can fire in bursts; one check per this long. */
export const MIN_CHECK_GAP_MS = 30_000;
const SNOOZE_KEY = 'mw.newVersion.snooze';

export interface NewVersionDeps {
  /** The server's current build id, or null when unavailable. */
  fetchId: () => Promise<string | null>;
  /** Signed in and the setting is on. */
  active: () => boolean;
  /** Show the toast; `later` is its close, `reload` its action. */
  show: (handlers: { reload: () => void; later: () => void }) => void;
  hide: () => void;
  now?: () => number;
  random?: () => number;
  storage?: Pick<Storage, 'getItem' | 'setItem'> | null;
  snoozeMs?: number;
  pollMs?: number;
}

export interface NewVersionNotice {
  start(): void;
  stop(): void;
  /** Check now (subject to the gap unless `force`). */
  check(force?: boolean): Promise<void>;
  /** The build this page booted with, once known. */
  booted(): string | null;
}

function session(): Storage | null {
  try {
    return typeof sessionStorage === 'undefined' ? null : sessionStorage;
  } catch {
    return null;
  }
}

export function createNewVersionNotice(deps: NewVersionDeps): NewVersionNotice {
  const now = deps.now ?? Date.now;
  const random = deps.random ?? Math.random;
  const snoozeMs = deps.snoozeMs ?? SNOOZE_MS;
  const pollMs = deps.pollMs ?? POLL_MS;
  const store = deps.storage === undefined ? session() : deps.storage;
  let bootId: string | null = null;
  let shown: string | null = null;
  let lastCheck = -Infinity;
  let running = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  function snoozedUntil(id: string): number {
    try {
      const raw = store?.getItem(SNOOZE_KEY);
      if (!raw) return 0;
      const v = JSON.parse(raw) as { id?: unknown; until?: unknown };
      return v.id === id && typeof v.until === 'number' ? v.until : 0;
    } catch {
      return 0;
    }
  }
  function snooze(id: string): void {
    try {
      store?.setItem(SNOOZE_KEY, JSON.stringify({ id, until: now() + snoozeMs }));
    } catch {
      /* private mode: the in-memory `shown` guard still holds until reload */
    }
  }

  async function check(force = false): Promise<void> {
    if (!deps.active()) return;
    if (!force && now() - lastCheck < MIN_CHECK_GAP_MS) return;
    lastCheck = now();
    const mine = ++seq;
    const id = await deps.fetchId();
    if (mine !== seq || id === null || !deps.active()) return;
    if (bootId === null) {
      bootId = id;
      return;
    }
    if (id === bootId) {
      if (shown !== null) {
        shown = null;
        deps.hide();
      }
      return;
    }
    if (shown === id || now() < snoozedUntil(id)) return;
    shown = id;
    deps.show({
      reload: () => {
        shown = null;
      },
      later: () => {
        snooze(id);
        shown = null;
      },
    });
  }

  function onVisible(): void {
    if (typeof document === 'undefined' || document.visibilityState === 'visible') void check();
  }
  const onFocus = (): void => void check();
  const onOnline = (): void => void check(true);

  function schedule(): void {
    if (!running) return;
    timer = setTimeout(() => {
      void check(true).finally(schedule);
    }, pollMs * (0.85 + random() * 0.3));
  }

  return {
    start() {
      if (running) return;
      running = true;
      window.addEventListener('focus', onFocus);
      window.addEventListener('online', onOnline);
      document.addEventListener('visibilitychange', onVisible);
      void check(true);
      schedule();
    },
    stop() {
      running = false;
      clearTimeout(timer);
      window.removeEventListener('focus', onFocus);
      window.removeEventListener('online', onOnline);
      document.removeEventListener('visibilitychange', onVisible);
      seq++;
      if (shown !== null) deps.hide();
      shown = null;
      bootId = null;
      lastCheck = -Infinity;
    },
    check,
    booted: () => bootId,
  };
}
