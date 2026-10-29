import { createEffect, createSignal, onCleanup } from 'solid-js';
import { parseHash, type ShellRoute } from '../shell/router.ts';
import { observePreferenceAccount, type PreferenceAccount } from './accountPreferenceLifecycle.ts';
import { accountUiPreferences, accountUiPreferenceStatus, setAccountUiPreferences } from './accountPreferences.ts';
import { analyticsPrivacySignal, analyticsScreen, createAnalyticsReporter } from './analytics.ts';

/**
 * The current shell route, observed READ-ONLY. Analytics must not route: a
 * second `createShellRouter` here replaced an unknown route (`#/bogus`) with the
 * inbox before the shell's own router saw it, so the shell never said why the
 * page changed (t32-h10), and its hashchange handler rewrote the hash beside
 * the shell's. Only the closed surface enum is read; parameters never are.
 */
export function observeShellRoute(): () => ShellRoute {
  const read = (): ShellRoute => parseHash(typeof location === 'undefined' ? '' : location.hash);
  const [route, setRoute] = createSignal<ShellRoute>(read());
  if (typeof window !== 'undefined') {
    const onHash = (): void => { setRoute(read()); };
    window.addEventListener('hashchange', onHash);
    onCleanup(() => window.removeEventListener('hashchange', onHash));
  }
  return route;
}

/** The consent key this document's browser analytics tag was served for (t63),
 *  or `null` when the server served no tag. It writes `<meta name="mw-analytics"
 *  data-consent="<provider>:<sha256>">` only into a consenting user's mail shell. */
export function documentAnalyticsTag(doc: Document = document): string | null {
  const key = doc.querySelector('meta[name="mw-analytics"]')?.getAttribute('data-consent');
  return key && /^[a-z-]+:[0-9a-f]{64}$/.test(key) ? key : null;
}

/**
 * Stop a running page from sending anything more off-origin: `sendBeacon`,
 * `fetch` and `XMLHttpRequest` to another origin fail from here on. Best effort
 * for the moment between a withdrawal and the reload. A provider script cannot be
 * unloaded, but it can be cut off before the page goes away, including its
 * pagehide or unload beacon when it uses these APIs. A pixel `<img>` it creates
 * is not covered.
 */
export function severThirdPartyNetwork(win: Window & typeof globalThis = window): void {
  const own = win.location.origin;
  const foreign = (input: unknown): boolean => {
    try {
      const raw = typeof input === 'string' ? input : input instanceof URL ? input.href : (input as { url?: string } | null)?.url ?? '';
      return new URL(raw, win.location.href).origin !== own;
    } catch { return true; }
  };
  try { Object.defineProperty(win.navigator, 'sendBeacon', { configurable: true, value: () => false }); } catch { /* frozen navigator */ }
  const fetch = win.fetch.bind(win);
  win.fetch = ((input: RequestInfo | URL, init?: RequestInit) => foreign(input)
    ? Promise.reject(new TypeError('analytics consent withdrawn'))
    : fetch(input, init)) as typeof win.fetch;
  const open = win.XMLHttpRequest.prototype.open as (...args: unknown[]) => void;
  win.XMLHttpRequest.prototype.open = function (this: XMLHttpRequest, ...args: unknown[]) {
    if (foreign(args[1])) throw new TypeError('analytics consent withdrawn');
    open.apply(this, args);
  } as typeof win.XMLHttpRequest.prototype.open;
}

/**
 * A provider's script cannot be unloaded from a running page, and the CSP that
 * admits it is fixed for the document. So once the user withdraws consent, a
 * privacy signal appears, or the stored consent no longer names this document's
 * provider and script, the page is cut off from third-party network at once. It
 * reloads as soon as the withdrawal is saved (or the save fails: the page must
 * not keep running a tag whose consent this tab already withdrew), and the
 * server then serves the shell without the tag and without the CSP additions. A
 * different account in the same tab reloads at once, so a tag served for one
 * account never observes another.
 *
 * Other tabs of this browser hear about it on {@link ANALYTICS_CHANNEL} (t63 R4):
 * a withdrawal, another account signing in, or the admin console being used,
 * cuts off and reloads every tab that runs a tag.
 */
export function createTagGuard(served: string | null, reload: () => void, sever: () => void = () => {}, announce: () => void = () => {}, retry: (attempt: number) => void = () => {}) {
  let servedFor: string | null = null;
  let severed = false;
  let reloaded = false;
  const cut = (): void => { if (!severed) { severed = true; sever(); } };
  const reloadOnce = (): void => { if (!reloaded) { reloaded = true; reload(); } };
  let announced = false;
  // Has this tab seen the account's consent equal to this document's? Only
  // then can a non-matching value be this tab's own withdrawal.
  let seen = false;
  let attempts = 0;
  const announceOnce = (): void => { if (!announced) { announced = true; announce(); } };
  return {
    account(account: PreferenceAccount | null): void {
      if (account === null) return;
      const key = `${account.id}\n${account.storageAuthority ?? ''}`;
      if (servedFor === null) { servedFor = key; return; }
      if (key === servedFor) return;
      // Another account signed in here: tags in other tabs must not observe it.
      announceOnce();
      if (served !== null) { cut(); reloadOnce(); }
    },
    /** Another tab withdrew, signed in another account, or used the console. */
    revoked(): void {
      if (served === null) return;
      cut();
      reloadOnce();
    },
    /**
     * `consent` is the account's current tag consent (`undefined` while not
     * yet loaded) and `status` the preferences sync state (t63 T3):
     *  - `synced` and different from this document's: a saved withdrawal (here
     *    or elsewhere). Cut off, tell the other tabs, reload.
     *  - `saving` after this tab saw the consent: a local withdrawal on its way.
     *    Cut off now, reload once it is saved.
     *  - `error` after this tab saw the consent: that withdrawal failed to save.
     *    Stay cut off and retry the save with backoff (t63 F1). Reloading here
     *    would serve the tag again (the server still holds the consent) and,
     *    with the pending write replayed, loop. Other tabs are not told until
     *    the save succeeds.
     *  - `error` before the consent was ever seen is a failed LOAD, which says
     *    nothing about consent: the tag is left alone, so a persistent failure
     *    cannot reload-loop every tab.
     * A privacy signal cuts off and reloads this tab only.
     */
    consent(consent: string | undefined, privacySignal: boolean, status: string): void {
      if (served === null) return;
      if (privacySignal) { cut(); reloadOnce(); return; }
      if (consent === undefined) return;
      if (consent === served) { seen = true; return; }
      if (status === 'synced') { cut(); announceOnce(); reloadOnce(); return; }
      if (!seen) return;
      cut();
      if (status === 'error') retry(attempts++);
    },
  };
}

/** Same-browser tabs tell each other to drop a running analytics tag (t63 R4). */
export const ANALYTICS_CHANNEL = 'mw-analytics';

/** Tell every tab of this browser to drop a running analytics tag. Best effort:
 *  BroadcastChannel can be missing (old browsers, some private modes). */
export function announceAnalyticsRevoke(): void {
  try {
    const channel = new BroadcastChannel(ANALYTICS_CHANNEL);
    channel.postMessage({ type: 'revoke' });
    channel.close();
  } catch { /* nothing to tell */ }
}

/** The shared route supplies a closed surface enum; route parameters are never read. */
export function startAnalytics(): void {
  const route = observeShellRoute();
  const reporter = createAnalyticsReporter();
  const [account, setAccount] = createSignal<PreferenceAccount | null>(null);
  const [visible, setVisible] = createSignal(document.visibilityState === 'visible');
  // One channel object both listens and announces: a BroadcastChannel never
  // delivers a message to the object that posted it, so this tab does not
  // reload on its own withdrawal before the change is saved.
  let channel: BroadcastChannel | null = null;
  try { channel = new BroadcastChannel(ANALYTICS_CHANNEL); } catch { channel = null; }
  let retryTimer: ReturnType<typeof setTimeout> | undefined;
  const guard = createTagGuard(documentAnalyticsTag(), () => { location.reload(); }, () => { severThirdPartyNetwork(); },
    () => { try { channel?.postMessage({ type: 'revoke' }); } catch { /* closed */ } },
    // Save the withdrawal again: 1 s, 2 s, 4 s … up to a minute.
    attempt => {
      clearTimeout(retryTimer);
      retryTimer = setTimeout(() => { setAccountUiPreferences({ analyticsTagConsent: accountUiPreferences().analyticsTagConsent ?? '' }); }, Math.min(60_000, 1000 * 2 ** attempt));
    });
  if (channel) {
    channel.onmessage = (event: MessageEvent<unknown>) => {
      if ((event.data as { type?: unknown } | null)?.type === 'revoke') guard.revoked();
    };
  }
  const stopAccount = observePreferenceAccount(next => { guard.account(next); setAccount(next); });
  const visibility = (): void => { setVisible(document.visibilityState === 'visible'); };
  document.addEventListener('visibilitychange', visibility);
  createEffect(() => {
    reporter.update(account()?.generation ?? null, analyticsScreen(route().surface),
      visible() && accountUiPreferenceStatus() === 'synced' && accountUiPreferences().analyticsConsent === true,
      analyticsPrivacySignal());
  });
  createEffect(() => {
    const status = accountUiPreferenceStatus();
    // Known once loaded: a local withdrawal counts while it is still saving.
    const known = status === 'synced' || status === 'saving' || status === 'error';
    guard.consent(known ? accountUiPreferences().analyticsTagConsent ?? '' : undefined, analyticsPrivacySignal(), status);
  });
  // A tagged page restored from the back/forward cache may meet a browser that
  // has since been used for the console, or a withdrawn consent: reload it, so
  // the server decides again (t63 T4).
  const pageshow = (event: PageTransitionEvent): void => { if (event.persisted) guard.revoked(); };
  window.addEventListener('pageshow', pageshow);
  onCleanup(() => { clearTimeout(retryTimer); reporter.stop(); stopAccount(); channel?.close(); window.removeEventListener('pageshow', pageshow); document.removeEventListener('visibilitychange', visibility); });
}
