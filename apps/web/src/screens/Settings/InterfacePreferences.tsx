import {For, Show, onMount} from 'solid-js';
import {loadCatalog, t} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';

const fields: {key:string; fallback:string|number|boolean; choices?:string[]; min?:number; max?:number; color?:boolean}[] = [
  {key:'appearance.fontSize',fallback:14,min:12,max:24},
  {key:'appearance.fontWeight',fallback:'400',choices:['300','400','500','600']},
  {key:'appearance.fontColor',fallback:'',color:true},
  {key:'appearance.lowContrast',fallback:false},
  {key:'mail.messageListStyle',fallback:'cards',choices:['cards','table']},
  {key:'mail.messageListOrder',fallback:'newest',choices:['newest','oldest','sender','subject']},
  {key:'mail.messageListPreview',fallback:true},
  {key:'mail.afterDelete',fallback:'next',choices:['next','previous','close']},
  {key:'reader.attachmentsAtEnd',fallback:false},
  {key:'ui.notifyNewVersion',fallback:true},
];

export function InterfacePreferences() {
  onMount(()=>void loadCatalog('appearance'));
  return <div><For each={fields}>{field=>{
    const setting=useSetting<string|number|boolean>(field.key,field.fallback);
    const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
    const id=`preference-${field.key}`;
    return <div class={css.field}>
      <label for={id}>{t(`appearance-preference-${field.key.replaceAll('.', '-')}`)}</label>
      <Show when={typeof field.fallback !== 'boolean'} fallback={<input id={id} type="checkbox" checked={setting.value()===true} disabled={disabled()} onChange={event=>void setting.set(event.currentTarget.checked)}/> }>
        <Show when={field.choices} fallback={<input id={id} class={css.input} type={field.color?'color':'number'} min={field.min} max={field.max} step="1" value={field.color ? (setting.value() || '#000000') as string : setting.value() as number} disabled={disabled()} onChange={event=>{if(event.currentTarget.checkValidity()) void setting.set(field.color?event.currentTarget.value:event.currentTarget.valueAsNumber);}}/>}>
          {choices=><select id={id} class={css.select} value={String(setting.value())} disabled={disabled()} onChange={event=>void setting.set(event.currentTarget.value)}><For each={choices()}>{choice=><option value={choice}>{t(`appearance-preference-option-${choice}`)}</option>}</For></select>}
        </Show>
      </Show>
      <LockNote locked={setting.locked()}/>
      <Show when={setting.error()}><p role="alert">{setting.error()}</p></Show>
      <button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void setting.reset()}>{t('appearance-preference-inherit')}</button>
    </div>;
  }}</For></div>;
}
