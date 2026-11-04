import type { JSX } from 'solid-js';
import { t } from '../../../i18n';
import { InlineGroup, SettingsGate, rowsFor } from './InlineGroup.tsx';
import { FETCH_LIMIT_KEYS, fetchLimitValue, type PrivacyTabContext } from './model.ts';

export default function FetchLimitsTab(props: { ctx: PrivacyTabContext }): JSX.Element {
  return (
    <SettingsGate ctx={props.ctx}>
      <InlineGroup ctx={props.ctx} title={t('admin-settings-privacy-fetch-title')} description={t('admin-settings-privacy-fetch-help')} rows={rowsFor(props.ctx.rows(), FETCH_LIMIT_KEYS)} format={(s) => fetchLimitValue(s.key, s.effective.value)} />
    </SettingsGate>
  );
}
