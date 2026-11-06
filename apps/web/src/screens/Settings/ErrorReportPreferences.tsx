// The per-person switch for browser error reports (`privacy.errorReports`,
// observability/clientErrors.ts). On by default; an administrator can set the
// default, lock it, or switch reporting off for everyone
// (`observability.client.enabled`). The setting is enforced by the server on the
// signed-in route, so turning it off stops the reports even from a stale tab.

import { onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { cacheErrorReports } from '../../observability/clientErrors.ts';
import { useSetting } from './useSetting.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';

export function ErrorReportPreferences(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  const setting = useSetting<string>('privacy.errorReports', 'on', (v: unknown): v is string => v === 'on' || v === 'off');
  return <SettingsCard id="privacy-card-error-reports" testId="error-report-preferences" title={t('privacy-error-reports-title')} description={t('privacy-error-reports-help')}>
    <SettingRow controlId="error-reports-control" label={t('privacy-error-reports-send')} help={t('privacy-error-reports-send-help')} handle={setting} resetLabel={t('privacy-inherit')}>{describedBy =>
      <input id="error-reports-control" type="checkbox" data-testid="error-reports" aria-describedby={describedBy()}
        checked={setting.value() !== 'off'} disabled={!setting.entry() || setting.locked() || setting.saving()}
        onChange={event => { const next = event.currentTarget.checked ? 'on' : 'off'; cacheErrorReports(next); void setting.set(next); }} />
    }</SettingRow>
  </SettingsCard>;
}
