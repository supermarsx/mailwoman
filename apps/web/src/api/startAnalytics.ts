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

/** The shared route supplies a closed surface enum; route parameters are never read. */
export function startAnalytics(): void {
  const route = observeShellRoute();
  const reporter = createAnalyticsReporter();
  const [account, setAccount] = createSignal<PreferenceAccount | null>(null);
  const [visible, setVisible] = createSignal(document.visibilityState === 'visible');
  const stopAccount = observePreferenceAccount(setAccount);
  const visibility = (): void => { setVisible(document.visibilityState === 'visible'); };
  document.addEventListener('visibilitychange', visibility);
  createEffect(() => {
    reporter.update(account()?.generation ?? null, analyticsScreen(route().surface),
      visible() && accountUiPreferenceStatus() === 'synced' && accountUiPreferences().analyticsConsent === true,
      analyticsPrivacySignal());
  });
  onCleanup(() => { reporter.stop(); stopAccount(); document.removeEventListener('visibilitychange', visibility); });
}
