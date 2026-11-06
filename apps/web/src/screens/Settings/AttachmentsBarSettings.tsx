import { For, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';
import { DOWNLOAD_BUTTON_MODES, isDownloadButtonMode } from '../../components/attachmentsBarDownload.ts';

/** The "Attachments bar" card (Message display): how the reader's bar toggles and what downloads it offers.
 *  Start collapsed/expanded is the existing `reader.attachmentsCollapsed` row in the card above. */
const TOGGLES = [
  { id: 'reader-attachmentsBarClickToToggle', key: 'attachments.bar.clickToToggle', fallback: true },
  { id: 'reader-attachmentsBarDownloadAll', key: 'attachments.bar.downloadAll', fallback: true },
  { id: 'reader-attachmentsBarZipInlineImages', key: 'attachments.bar.zipInlineImages', fallback: false },
] as const;
const MODE_ID = 'reader-attachmentsBarShowDownloadButtons';
const MODE_KEY = 'attachments.bar.showDownloadButtons';

function ToggleRow(props: { id: string; settingKey: string; fallback: boolean }): JSX.Element {
  const setting = useSetting<boolean>(props.settingKey, props.fallback, (v): v is boolean => typeof v === 'boolean');
  return <SettingRow controlId={props.id} label={t(props.id)} help={t(`${props.id}-help`)} handle={setting}>{describedBy =>
    <input id={props.id} type="checkbox" checked={setting.value()} aria-describedby={describedBy()} disabled={!setting.entry() || setting.locked() || setting.saving()} onChange={event => void setting.set(event.currentTarget.checked)} />
  }</SettingRow>;
}

function ModeRow(): JSX.Element {
  const setting = useSetting<string>(MODE_KEY, 'hover', (v): v is string => isDownloadButtonMode(v));
  return <SettingRow controlId={MODE_ID} label={t(MODE_ID)} help={t(`${MODE_ID}-help`)} handle={setting}>{describedBy =>
    <select id={MODE_ID} class={css.select} value={setting.value()} aria-describedby={describedBy()} disabled={!setting.entry() || setting.locked() || setting.saving()} onChange={event => void setting.set(event.currentTarget.value)}>
      <For each={DOWNLOAD_BUTTON_MODES}>{mode => <option value={mode}>{t(`reader-option-${mode}`)}</option>}</For>
    </select>
  }</SettingRow>;
}

export function AttachmentsBarSettings(): JSX.Element {
  return <SettingsCard id="reader-card-attachments-bar" title={t('reader-card-attachments-bar')} description={t('reader-card-attachments-bar-help')}>
    <ToggleRow id={TOGGLES[0].id} settingKey={TOGGLES[0].key} fallback={TOGGLES[0].fallback} />
    <ModeRow />
    <ToggleRow id={TOGGLES[1].id} settingKey={TOGGLES[1].key} fallback={TOGGLES[1].fallback} />
    <ToggleRow id={TOGGLES[2].id} settingKey={TOGGLES[2].key} fallback={TOGGLES[2].fallback} />
  </SettingsCard>;
}

export const ATTACHMENTS_BAR_CARD_SPEC: CardSpec = {
  id: 'reader-card-attachments-bar', section: 'reader', title: 'reader-card-attachments-bar', help: 'reader-card-attachments-bar-help',
  rows: [TOGGLES[0].id, MODE_ID, TOGGLES[1].id, TOGGLES[2].id].map(id => ({ id, label: id, help: `${id}-help` })),
};
