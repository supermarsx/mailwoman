import {For, Show, onMount} from 'solid-js';
import {loadCatalog, t} from '../../i18n';
import {MESSAGE_LIST_GROUPS, MESSAGE_LIST_ORDERS} from '../../components/messageOrder.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import type {CardSpec} from './layout/rows.tsx';
import * as css from './styles.css.ts';

const fields: {key:string; fallback:string|number|boolean; choices?:readonly string[]; choicePrefix?:string; help?:string; min?:number; max?:number; color?:boolean}[] = [
  {key:'appearance.fontSize',fallback:14,min:12,max:24},
  {key:'appearance.fontWeight',fallback:'400',choices:['300','400','500','600']},
  {key:'appearance.fontColor',fallback:'',color:true},
  {key:'appearance.lowContrast',fallback:false},
  {key:'mail.messageListStyle',fallback:'cards',choices:['cards','table']},
  {key:'mail.messageListOrder',fallback:'newest',choices:MESSAGE_LIST_ORDERS,help:'appearance-preference-message-order-help'},
  {key:'mail.messageListGroup',fallback:'none',choices:MESSAGE_LIST_GROUPS,choicePrefix:'appearance-preference-group-option',help:'appearance-preference-message-group-help'},
  {key:'mail.messageListPreview',fallback:true},
  {key:'mail.list.pageSize',fallback:50,min:20,max:200},
  {key:'mail.afterDelete',fallback:'next',choices:['next','previous','close']},
  {key:'reader.attachmentsAtEnd',fallback:false},
  {key:'ui.notifyNewVersion',fallback:true},
];
const labelKey=(key:string):string=>`appearance-preference-${key.replaceAll('.', '-')}`;
/** Rows of the same card that Settings.tsx renders (not this component): density, font, layout, text mode. */
export const APPEARANCE_TEXT_ROWS=[
  {id:'settings-density',label:'settings-density'},
  {id:'settings-font',label:'settings-font'},
  {id:'settings-layout',label:'settings-layout'},
  {id:'settings-text-mode-label',label:'appearance-text-mode-label'},
] as const;
/** Search-index entry: these rows render inside the Appearance text card. */
export const INTERFACE_CARD:CardSpec={id:'settings-appearance-text-card',section:'appearance',title:'settings-appearance-text-card',help:'settings-appearance-text-card-help',rows:[...fields.map(f=>({id:`preference-${f.key}`,label:labelKey(f.key),...(f.help?{help:f.help}:{})})),...APPEARANCE_TEXT_ROWS]};

export function InterfacePreferences() {
  onMount(()=>void loadCatalog('appearance'));
  return <div><For each={fields}>{field=>{
    const setting=useSetting<string|number|boolean>(field.key,field.fallback);
    const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
    const id=`preference-${field.key}`;
    return <SettingRow controlId={id} label={t(labelKey(field.key))} {...(field.help?{help:t(field.help)}:{})} handle={setting} resetLabel={t('appearance-preference-inherit')}>{describedBy=>
      <Show when={typeof field.fallback !== 'boolean'} fallback={<input id={id} type="checkbox" checked={setting.value()===true} disabled={disabled()} aria-describedby={describedBy()} onChange={event=>void setting.set(event.currentTarget.checked)}/> }>
        <Show when={field.choices} fallback={<input id={id} class={css.input} type={field.color?'color':'number'} min={field.min} max={field.max} step="1" aria-describedby={describedBy()} value={field.color ? (setting.value() || '#000000') as string : setting.value() as number} disabled={disabled()} onChange={event=>{if(event.currentTarget.checkValidity()) void setting.set(field.color?event.currentTarget.value:event.currentTarget.valueAsNumber);}}/>}>
          {choices=><select id={id} class={css.select} value={String(setting.value())} disabled={disabled()} aria-describedby={describedBy()} onChange={event=>void setting.set(event.currentTarget.value)}><For each={choices()}>{choice=><option value={choice}>{t(`${field.choicePrefix??'appearance-preference-option'}-${choice}`)}</option>}</For></select>}
        </Show>
      </Show>
    }</SettingRow>;
  }}</For></div>;
}
