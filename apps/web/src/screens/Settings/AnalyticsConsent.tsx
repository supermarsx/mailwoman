import { createResource, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { accountUiPreferences, accountUiPreferenceStatus, setAccountUiPreferences } from '../../api/accountPreferences.ts';
import { analyticsPrivacySignal } from '../../api/analytics.ts';
import { loadBrowserTagStatus, TAG_PROVIDER_NAMES, type BrowserTagStatus } from '../../api/analyticsTagConsent.ts';
import { ControlRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';

export function AnalyticsConsent(props: { loadTag?: () => Promise<BrowserTagStatus | null> } = {}): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  const busy = (): boolean => ['idle','loading'].includes(accountUiPreferenceStatus());
  const [tag] = createResource(() => (props.loadTag ?? loadBrowserTagStatus)());
  return <SettingsCard id="privacy-card-analytics" title={t('privacy-analytics-title')} description={t('privacy-analytics-help')}>
    <ControlRow controlId="analytics-consent" label={t('privacy-analytics-consent')}>{() =>
    <input id="analytics-consent" type="checkbox" aria-describedby="privacy-card-analytics-desc" disabled={busy() || (analyticsPrivacySignal() && accountUiPreferences().analyticsConsent !== true)} checked={accountUiPreferences().analyticsConsent === true} onChange={event => setAccountUiPreferences({analyticsConsent:event.currentTarget.checked})} />
    }</ControlRow>
    <Show when={tag()}>{current => {
      // t63: a browser tag is a separate decision, bound to this provider and
      // this exact script. Consent to counts above never implies it, and a
      // change of provider or script asks again.
      const name = (): string => TAG_PROVIDER_NAMES[current().provider] ?? t('privacy-analytics-tag-custom');
      const given = (): boolean => accountUiPreferences().analyticsTagConsent === current().consentKey;
      const stale = (): boolean => !given() && (accountUiPreferences().analyticsTagConsent ?? '') !== '';
      return <div data-testid="analytics-tag-consent">
        <ControlRow controlId="analytics-tag-consent" label={t('privacy-analytics-tag-consent', { provider: name() })} help={t('privacy-analytics-tag-help', { provider: name() })}
          note={<Show when={stale()}><p class={css.meta} role="status">{t('privacy-analytics-tag-changed')}</p></Show>}>{(describedBy) =>
        <input id="analytics-tag-consent" type="checkbox" aria-describedby={describedBy()} disabled={busy() || (analyticsPrivacySignal() && !given())} checked={given()} onChange={event => setAccountUiPreferences({analyticsTagConsent: event.currentTarget.checked ? current().consentKey : ''})} />
        }</ControlRow>
      </div>;
    }}</Show>
    <Show when={analyticsPrivacySignal()}><p class={css.meta}>{t('privacy-analytics-signal')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'error'}><p role="alert">{t('privacy-analytics-save-error')}</p></Show>
    <Show when={accountUiPreferenceStatus() === 'saving'}><p role="status">{t('privacy-analytics-saving')}</p></Show>
  </SettingsCard>;
}
