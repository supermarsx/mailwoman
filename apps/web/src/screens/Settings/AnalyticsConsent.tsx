import { createResource, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { accountUiPreferences, accountUiPreferenceStatus, setAccountUiPreferences } from '../../api/accountPreferences.ts';
import { analyticsPrivacySignal } from '../../api/analytics.ts';
import { loadBrowserTagStatus, TAG_PROVIDER_NAMES, type BrowserTagStatus } from '../../api/analyticsTagConsent.ts';
import * as css from './styles.css.ts';

export function AnalyticsConsent(props: { loadTag?: () => Promise<BrowserTagStatus | null> } = {}): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  const busy = (): boolean => ['idle','loading'].includes(accountUiPreferenceStatus());
  const [tag] = createResource(() => (props.loadTag ?? loadBrowserTagStatus)());
  return <section class={css.section} aria-labelledby="analytics-consent-heading">
    <h3 id="analytics-consent-heading" class={css.heading}>{t('privacy-analytics-title')}</h3>
    <p class={css.prose} id="analytics-consent-help">{t('privacy-analytics-help')}</p>
    <label><input type="checkbox" aria-describedby="analytics-consent-help" disabled={busy() || (analyticsPrivacySignal() && accountUiPreferences().analyticsConsent !== true)} checked={accountUiPreferences().analyticsConsent === true} onChange={event => setAccountUiPreferences({analyticsConsent:event.currentTarget.checked})} /> {t('privacy-analytics-consent')}</label>
    <Show when={tag()}>{current => {
      // t63: a browser tag is a separate decision, bound to this provider and
      // this exact script. Consent to counts above never implies it, and a
      // change of provider or script asks again.
      const name = (): string => TAG_PROVIDER_NAMES[current().provider] ?? t('privacy-analytics-tag-custom');
      const given = (): boolean => accountUiPreferences().analyticsTagConsent === current().consentKey;
      const stale = (): boolean => !given() && (accountUiPreferences().analyticsTagConsent ?? '') !== '';
      return <div data-testid="analytics-tag-consent">
        <h4 class={css.heading}>{t('privacy-analytics-tag-title', { provider: name() })}</h4>
        <p class={css.prose} id="analytics-tag-consent-help">{t('privacy-analytics-tag-help', { provider: name() })}</p>
        <Show when={stale()}><p class={css.meta} role="status">{t('privacy-analytics-tag-changed')}</p></Show>
        <label><input type="checkbox" aria-describedby="analytics-tag-consent-help" disabled={busy() || (analyticsPrivacySignal() && !given())} checked={given()} onChange={event => setAccountUiPreferences({analyticsTagConsent: event.currentTarget.checked ? current().consentKey : ''})} /> {t('privacy-analytics-tag-consent', { provider: name() })}</label>
      </div>;
    }}</Show>
    <Show when={analyticsPrivacySignal()}><p class={css.meta}>{t('privacy-analytics-signal')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'error'}><p role="alert">{t('privacy-analytics-save-error')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'saving'}><p role="status">{t('privacy-analytics-saving')}</p></Show>
  </section>;
}
