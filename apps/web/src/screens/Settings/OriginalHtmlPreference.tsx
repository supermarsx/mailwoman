import { onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { featureOn, featuresLoaded } from '../../state/features.ts';
import { useSetting } from './useSetting.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';

/** The opt-in never selects original HTML automatically or overrides administrator policy. */
export function OriginalHtmlPreference(): JSX.Element {
  onMount(() => { void loadCatalog('original-html-settings'); void loadCatalog('privacy'); });
  const allowed = useSetting('mail.reader.allowOriginalHtml', false);
  const capability = useSetting('cap.reading.original_html', true);
  const handling = useSetting<string>('mail.reader.unsafeHtmlMode', 'sanitized', (value): value is string => typeof value === 'string' && ['ask', 'strict', 'sanitized'].includes(value));
  const available = (): boolean => featuresLoaded() && featureOn('reading.original_html') && capability.value() && !(handling.value() === 'strict' && handling.locked());
  const disabled = (): boolean => !allowed.entry() || allowed.locked() || allowed.saving() || !available();
  const help = (): string => {
    const explanation = !featuresLoaded() ? 'original-html-settings-loading'
      : !featureOn('reading.original_html') ? 'original-html-settings-feature-off'
      : !capability.value() ? 'original-html-settings-capability-off'
      : handling.value() === 'strict' && handling.locked() ? 'original-html-settings-strict'
      : 'original-html-settings-use';
    return `${t('privacy-original-html-help')} ${t(explanation)}`;
  };
  return <SettingsCard id="privacy-card-original-html" title={t('original-html-settings-title')} description={t('original-html-settings-help')}>
    <SettingRow controlId="privacy-original-html" label={t('privacy-original-html-allow')} help={help()} handle={allowed} lockNote={allowed.locked() || featuresLoaded() && !available()} resetLabel={t('privacy-inherit')} resetDisabled={disabled()} onReset={() => { if (available()) void allowed.reset(); }}>{describedBy =>
      <input id="privacy-original-html" type="checkbox" checked={available() && allowed.value()} disabled={disabled()} aria-describedby={describedBy()} onChange={event => { if (available()) void allowed.set(event.currentTarget.checked); }} />
    }</SettingRow>
  </SettingsCard>;
}
