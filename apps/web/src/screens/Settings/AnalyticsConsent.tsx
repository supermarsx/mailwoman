import { onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { accountUiPreferences, accountUiPreferenceStatus, setAccountUiPreferences } from '../../api/accountPreferences.ts';
import { analyticsPrivacySignal } from '../../api/analytics.ts';
import * as css from './styles.css.ts';

export function AnalyticsConsent(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  const busy = (): boolean => ['idle','loading'].includes(accountUiPreferenceStatus());
  return <section class={css.section} aria-labelledby="analytics-consent-heading">
    <h3 id="analytics-consent-heading" class={css.heading}>{t('privacy-analytics-title')}</h3>
    <p class={css.prose} id="analytics-consent-help">{t('privacy-analytics-help')}</p>
    <label><input type="checkbox" aria-describedby="analytics-consent-help" disabled={busy() || (analyticsPrivacySignal() && accountUiPreferences().analyticsConsent !== true)} checked={accountUiPreferences().analyticsConsent === true} onChange={event => setAccountUiPreferences({analyticsConsent:event.currentTarget.checked})} /> {t('privacy-analytics-consent')}</label>
    <Show when={analyticsPrivacySignal()}><p class={css.meta}>{t('privacy-analytics-signal')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'error'}><p role="alert">{t('privacy-analytics-save-error')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'saving'}><p role="status">{t('privacy-analytics-saving')}</p></Show>
  </section>;
}
