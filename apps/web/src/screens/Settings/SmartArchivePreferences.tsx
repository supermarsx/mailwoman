import {For,Show,type JSX} from 'solid-js';
import {SMART_ARCHIVE_DEFAULTS,SMART_ARCHIVE_STRATEGIES,type SmartArchivePolicy} from '../../state/smartArchive.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
interface Field {key:keyof SmartArchivePolicy;type:'boolean'|'enum'|'number'|'folders';options?:readonly string[]}
const fields:readonly Field[]=[{key:'enabled',type:'boolean'},{key:'replaceArchive',type:'boolean'},{key:'strategy',type:'enum',options:SMART_ARCHIVE_STRATEGIES},{key:'multipleFolders',type:'enum',options:['ask','latest','most-used']},{key:'confirmation',type:'enum',options:['always','when-needed','never']},{key:'fallback',type:'enum',options:['archive','choose','cancel']},{key:'historyLimit',type:'number'},{key:'includeSubdomains',type:'boolean'},{key:'excludedFolders',type:'folders'}];
export const SMART_ARCHIVE_CARD:CardSpec={id:'settings-card-smart-archive',section:'reader',title:'smart-archive-title',help:'smart-archive-help',rows:fields.map(field=>({id:`smart-archive-${field.key}`,label:`smart-archive-${field.key}`,help:`smart-archive-${field.key}-help`}))};
function Row(props:{field:Field}):JSX.Element{
 const field=props.field,id=`smart-archive-${field.key}`,fallback=SMART_ARCHIVE_DEFAULTS[field.key];
 const handle=useSetting<unknown>(`mail.smartArchive.${field.key}`,fallback,(value):value is unknown=>Array.isArray(fallback)?Array.isArray(value):typeof value===typeof fallback);
 const disabled=()=>!handle.entry()||handle.locked()||handle.saving();
 return <SettingRow controlId={id} label={t(id)} help={t(`${id}-help`)} handle={handle}>{description=><>
  <Show when={field.type==='boolean'}><input id={id} type="checkbox" checked={handle.value()===true} disabled={disabled()} aria-describedby={description()} onChange={event=>void handle.set(event.currentTarget.checked)}/></Show>
  <Show when={field.type==='enum'}><select id={id} class={css.select} value={String(handle.value())} disabled={disabled()} aria-describedby={description()} onChange={event=>void handle.set(event.currentTarget.value)}><For each={field.options}>{option=><option value={option}>{t(`smart-archive-option-${field.key}-${option}`)}</option>}</For></select></Show>
  <Show when={field.type==='number'}><input id={id} class={css.input} type="number" min="10" max="500" value={Number(handle.value())} disabled={disabled()} aria-describedby={description()} onChange={event=>{const value=event.currentTarget.valueAsNumber;if(Number.isInteger(value)&&value>=10&&value<=500)void handle.set(value);}}/></Show>
  <Show when={field.type==='folders'}><textarea id={id} class={css.input} value={(handle.value() as string[]).join('\n')} disabled={disabled()} aria-describedby={description()} onChange={event=>{const values=event.currentTarget.value.split(/\r?\n/).map(value=>value.trim()).filter(Boolean);if(values.length<=100&&values.every(value=>/^(role|id):[^\r\n]+$/.test(value)))void handle.set(values);}}/></Show>
 </>}</SettingRow>;
}
export function SmartArchivePreferences():JSX.Element{return <SettingsCard id={SMART_ARCHIVE_CARD.id} title={t('smart-archive-title')} description={t('smart-archive-help')}><For each={fields}>{field=><Row field={field}/>}</For></SettingsCard>;}
