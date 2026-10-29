import { createEffect, createSignal, onCleanup } from 'solid-js';
import { parseHash, type ShellRoute } from '../shell/router.ts';
import { observePreferenceAccount, type PreferenceAccount } from './accountPreferenceLifecycle.ts';
import { accountUiPreferences, accountUiPreferenceStatus } from './accountPreferences.ts';
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
 * reloads as soon as the withdrawal is saved, and the server then serves the
 * shell without the tag and without the CSP additions. A different account in
 * the same tab reloads at once, so a tag served for one account never observes
 * another.
 */
export function createTagGuard(served: string | null, reload: () => void, sever: () => void = () => {}) {
  let servedFor: string | null = null;
  let severed = false;
  let reloaded = false;
  const cut = (): void => { if (!severed) { severed = true; sever(); } };
  const reloadOnce = (): void => { if (!reloaded) { reloaded = true; reload(); } };
  return {
    account(account: PreferenceAccount | null): void {
      if (served === null || account === null) return;
      const key = `${account.id}\n${account.storageAuthority ?? ''}`;
      if (servedFor === null) servedFor = key;
      else if (key !== servedFor) { cut(); reloadOnce(); }
    },
    /** `consent` is the account's current tag consent (`undefined` while not yet
     *  loaded); `persisted` is whether the account's preferences are saved. */
    consent(consent: string | undefined, privacySignal: boolean, persisted: boolean): void {
      if (served === null || (consent === undefined && !privacySignal)) return;
      if (consent === served && !privacySignal) return;
      cut();
      if (persisted || privacySignal) reloadOnce();
    },
  };
}

/** The shared route supplies a closed surface enum; route parameters are never read. */
export function startAnalytics(): void {
  const route = observeShellRoute();
  const reporter = createAnalyticsReporter();
  const [account, setAccount] = createSignal<PreferenceAccount | null>(null);
  const [visible, setVisible] = createSignal(document.visibilityState === 'visible');
  const guard = createTagGuard(documentAnalyticsTag(), () => { location.reload(); }, () => { severThirdPartyNetwork(); });
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
    guard.consent(known ? accountUiPreferences().analyticsTagConsent ?? '' : undefined, analyticsPrivacySignal(), status === 'synced');
  });
  onCleanup(() => { reporter.stop(); stopAccount(); document.removeEventListener('visibilitychange', visibility); });
}
