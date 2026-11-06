import {useApp} from '../../state/context.ts';
import {For,Show,type JSX} from 'solid-js';
import {MAIL_CACHE_FIELDS,DEFAULT_MAIL_CACHE_POLICY} from '../../state/mailCachePolicy.ts';
import {useSetting} from './useSetting.ts';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import type {CardSpec} from './layout/rows.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';

type Field=typeof MAIL_CACHE_FIELDS[number];
const STORE_KEYS=['enabled','refreshOnOpen','mailboxes','messages','freshSeconds','maxAgeSeconds','maxEntries','maxBytes'];
const PREFETCH_KEYS=MAIL_CACHE_FIELDS.map(f=>f[0]).filter(k=>!STORE_KEYS.includes(k));
/** Search-index entries (offline / cache section). */
export const CACHE_CARDS:readonly CardSpec[]=[
 {id:'cache-card-store',section:'reader',title:'cache-card-store',help:'cache-help',rows:[...STORE_KEYS.map(k=>({id:`cache-${k}`,label:`cache-${k}`})),{id:'cache-excludedFolders',label:'cache-excludedFolders',help:'cache-exclusions-help'}]},
 {id:'cache-card-prefetch',section:'reader',title:'cache-card-prefetch',help:'cache-card-prefetch-help',rows:PREFETCH_KEYS.map(k=>({id:`cache-${k}`,label:`cache-${k}`}))},
];

function CacheRow(props:{field:Field}):JSX.Element{
 const field=props.field;
 const setting=useSetting<boolean|number>(`mail.cache.${field[0]}`,field[1],(value):value is boolean|number=>typeof value===typeof field[1]);
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 const id=`cache-${field[0]}`;
 return <SettingRow controlId={id} label={t(id)} handle={setting}>{d=>
  <Show when={typeof field[1]==='boolean'} fallback={<input id={id} class={css.input} type="number" min={field.length>2?field[2]:undefined} max={field.length>2?field[3]:undefined} aria-describedby={d()} value={Number(setting.value())} disabled={disabled()} onChange={event=>{const value=event.currentTarget.valueAsNumber;if(typeof field[2]==='number'&&typeof field[3]==='number'&&Number.isInteger(value)&&value>=field[2]&&value<=field[3])void setting.set(value);}}/>}>
   <input id={id} type="checkbox" checked={setting.value()===true} disabled={disabled()} aria-describedby={d()} onChange={event=>void setting.set(event.currentTarget.checked)}/>
  </Show>}</SettingRow>;
}

export function CachePreferences():JSX.Element{
 const app=useApp();
 const exclusions=useSetting<string[]>('mail.cache.excludedFolders',[...DEFAULT_MAIL_CACHE_POLICY.excludedFolders],(value):value is string[]=>Array.isArray(value)&&value.every(v=>typeof v==='string'));
 const fieldsFor=(keys:string[])=>MAIL_CACHE_FIELDS.filter(f=>keys.includes(f[0]));
 return <>
  <SettingsCard id="cache-card-store" title={t('cache-card-store')} description={t('cache-help')}>
   <div><button type="button" class={css.ghost} onClick={()=>app.clearMailCache()}>{t('cache-clear')}</button></div>
   <For each={fieldsFor(STORE_KEYS)}>{field=><CacheRow field={field}/>}</For>
   <SettingRow stacked controlId="cache-excludedFolders" label={t('cache-excludedFolders')} help={t('cache-exclusions-help')} handle={exclusions}>{d=>
    <textarea id="cache-excludedFolders" class={css.input} aria-describedby={d()} value={exclusions.value().join('\n')} disabled={!exclusions.entry()||exclusions.locked()||exclusions.saving()} onChange={event=>{const value=event.currentTarget.value.split(/\r?\n/).map(v=>v.trim()).filter(Boolean);void exclusions.set(value);}}/>}</SettingRow>
   <Show when={exclusions.error()}><p role="alert">{exclusions.error()}</p></Show>
  </SettingsCard>
  <SettingsCard id="cache-card-prefetch" title={t('cache-card-prefetch')} description={t('cache-card-prefetch-help')}>
   <For each={fieldsFor(PREFETCH_KEYS)}>{field=><CacheRow field={field}/>}</For>
  </SettingsCard>
 </>;
}
