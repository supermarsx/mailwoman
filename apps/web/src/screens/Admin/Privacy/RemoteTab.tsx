import type { JSX } from 'solid-js';
import { t } from '../../../i18n';
import { InlineGroup, SettingsGate, rowsFor } from './InlineGroup.tsx';
import { REMOTE_KEYS, type PrivacyTabContext } from './model.ts';

export default function RemoteTab(props: { ctx: PrivacyTabContext }): JSX.Element {
  return (
    <SettingsGate ctx={props.ctx}>
      <InlineGroup ctx={props.ctx} title={t('privacy-policy-values')} description={t('privacy-tab-remote-help')} rows={rowsFor(props.ctx.rows(), REMOTE_KEYS)} />
    </SettingsGate>
  );
}
