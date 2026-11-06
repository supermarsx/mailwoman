import {For,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {BADGE_DENSITIES,BADGE_DENSITY_KEY,BADGE_IDS,BADGE_LABELS,BADGE_LABELS_KEY,BADGE_MODES,BADGE_MODE_DEFAULTS,badgeKey,isBadgeDensity,isBadgeLabels,isBadgeMode,type BadgeDensity,type BadgeId,type BadgeLabels,type BadgeMode} from '../../state/readerBadges.ts';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
// Which badges the reader's header row shows (state/readerBadges.ts), in Settings > Message display.
export const MESSAGE_BADGES_CARD:CardSpec={id:'settings-card-message-badges',section:'reader',title:'settings-message-badges',help:'settings-message-badges-help',
 rows:[...BADGE_IDS.map(id=>({id:badgeKey(id),label:`settings-badge-${id}`,help:`settings-badge-${id}-help`})),{id:BADGE_DENSITY_KEY,label:'settings-badge-density',help:'settings-badge-density-help'},{id:BADGE_LABELS_KEY,label:'settings-badge-labels',help:'settings-badge-labels-help'}]};
function Choice<T extends string>(props:{id:string;label:string;help:string;values:readonly T[];name:(v:T)=>string;handle:ReturnType<typeof useSetting<T>>}):JSX.Element {
 const disabled=()=>!props.handle.entry()||props.handle.locked()||props.handle.saving();
 return <SettingRow controlId={props.id} label={props.label} help={props.help} handle={props.handle}>{describedBy=>
  <select id={props.id} class={css.select} value={props.handle.value()} disabled={disabled()} aria-describedby={describedBy()} onChange={event=>void props.handle.set(event.currentTarget.value as T)}><For each={props.values}>{v=><option value={v}>{props.name(v)}</option>}</For></select>}</SettingRow>;
}
function BadgeRow(props:{id:BadgeId}):JSX.Element {
 const handle=useSetting<BadgeMode>(badgeKey(props.id),BADGE_MODE_DEFAULTS[props.id],isBadgeMode);
 return <Choice id={badgeKey(props.id)} label={t(`settings-badge-${props.id}`)} help={t(`settings-badge-${props.id}-help`)} values={BADGE_MODES} name={v=>t(`settings-badge-mode-${v}`)} handle={handle}/>;
}
export function MessageBadges():JSX.Element {
 const density=useSetting<BadgeDensity>(BADGE_DENSITY_KEY,'compact',isBadgeDensity);
 const labels=useSetting<BadgeLabels>(BADGE_LABELS_KEY,'text',isBadgeLabels);
 return <SettingsCard id="settings-card-message-badges" title={t('settings-message-badges')} description={t('settings-message-badges-help')}>
  <For each={BADGE_IDS}>{id=><BadgeRow id={id}/>}</For>
  <Choice id={BADGE_DENSITY_KEY} label={t('settings-badge-density')} help={t('settings-badge-density-help')} values={BADGE_DENSITIES} name={v=>t(`settings-badge-density-${v}`)} handle={density}/>
  <Choice id={BADGE_LABELS_KEY} label={t('settings-badge-labels')} help={t('settings-badge-labels-help')} values={BADGE_LABELS} name={v=>t(`settings-badge-labels-${v}`)} handle={labels}/>
 </SettingsCard>;
}
