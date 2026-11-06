// Per-person switches for the folder and message load-time measurements
// (perf/clientTiming.ts). Both are on by default; an administrator can set the
// default or lock it, and a locked switch shows the "set by your administrator" note.
// Turning one off stops the beacon at once, with no reload.

import { onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import { KIND_SETTING, type TimingKind } from '../../perf/clientTiming.ts';

function KindSwitch(props: { kind: TimingKind }): JSX.Element {
  const setting = useSetting<boolean>(KIND_SETTING[props.kind], true);
  return <SettingRow controlId={`load-time-${props.kind}-control`} label={t(`privacy-load-time-${props.kind}`)} help={t(`privacy-load-time-${props.kind}-help`)} handle={setting} resetLabel={t('privacy-inherit')}>{describedBy =>
    <input id={`load-time-${props.kind}-control`} type="checkbox" data-testid={`load-time-${props.kind}`} aria-describedby={describedBy()}
      checked={setting.value()} disabled={!setting.entry() || setting.locked() || setting.saving()}
      onChange={event => void setting.set(event.currentTarget.checked)} />
  }</SettingRow>;
}

export function LoadTimePreferences(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); });
  return <SettingsCard id="privacy-card-load-time" testId="load-time-preferences" title={t('privacy-load-time-title')} description={t('privacy-load-time-help')}>
    <KindSwitch kind="folder" />
    <KindSwitch kind="message" />
  </SettingsCard>;
}
