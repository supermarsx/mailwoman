import { createEffect, createSignal, onCleanup } from 'solid-js';
import { createShellRouter } from '../shell/router.ts';
import { observePreferenceAccount, type PreferenceAccount } from './accountPreferenceLifecycle.ts';
import { accountUiPreferences, accountUiPreferenceStatus } from './accountPreferences.ts';
import { analyticsPrivacySignal, analyticsScreen, createAnalyticsReporter } from './analytics.ts';

/** The shared router supplies a closed surface enum; route parameters are never read. */
export function startAnalytics(): void {
  const router = createShellRouter();
  const reporter = createAnalyticsReporter();
  const [account, setAccount] = createSignal<PreferenceAccount | null>(null);
  const [visible, setVisible] = createSignal(document.visibilityState === 'visible');
  const stopAccount = observePreferenceAccount(setAccount);
  const visibility = (): void => { setVisible(document.visibilityState === 'visible'); };
  document.addEventListener('visibilitychange', visibility);
  createEffect(() => {
    reporter.update(account()?.generation ?? null, analyticsScreen(router.route().surface),
      visible() && accountUiPreferenceStatus() === 'synced' && accountUiPreferences().analyticsConsent === true,
      analyticsPrivacySignal());
  });
  onCleanup(() => { reporter.stop(); stopAccount(); document.removeEventListener('visibilitychange', visibility); });
}
