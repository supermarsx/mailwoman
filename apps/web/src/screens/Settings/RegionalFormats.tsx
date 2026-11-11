import {createEffect,createMemo,createSignal,For,Show,type JSX} from 'solid-js';
import {t,loadCatalog} from '../../i18n';
import {activeLocaleSignal} from '../../i18n/registry.ts';
import {configuredDateParts,DATE_FORMATS,TIME_FORMATS,regionalLocale,regionalOptions,regionalValue,validRegionalFormat,type RegionalKey} from '../../i18n/regional.ts';
import {useSetting} from './useSetting.ts';
import {SelectRow,type CardSpec} from './layout/rows.tsx';
import {SettingRow} from './layout/SettingRow.tsx';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {useLeaveConfirm} from '../../components/confirm.tsx';
import type {SettingsNavigation} from './navigation.tsx';
import * as css from './styles.css.ts';

const SAMPLE=new Date('2026-11-23T21:05:09Z');
function preview(changes:Partial<Record<RegionalKey,string>>={}):string {
 const locale=regionalLocale(activeLocaleSignal(),changes.region??regionalValue('region'));
 const make=(options:Intl.DateTimeFormatOptions)=>new Intl.DateTimeFormat(locale,options);
 const options:Intl.DateTimeFormatOptions={dateStyle:'medium',timeStyle:'short',timeZone:'UTC'};
 return (configuredDateParts(SAMPLE,options,make,changes)??make(regionalOptions(options,changes.hourCycle??regionalValue('hourCycle'))).formatToParts(SAMPLE)).map(part=>part.value).join('');
}
function FormatField(props:{kind:'region'|'dateFormat'|'timeFormat';navigation:SettingsNavigation}):JSX.Element {
 const setting=useSetting<string>(`time.${props.kind}`,'auto');
 const [draft,setDraft]=createSignal(setting.value()),[error,setError]=createSignal('');
 const saved=createMemo(()=>setting.value());createEffect(()=>setDraft(saved()));
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving();
 const LeaveDialog=useLeaveConfirm(props.navigation,()=>draft()!==setting.value(),()=>t('regional-discard'));
 const regions=createMemo(()=>{const names=new Intl.DisplayNames(activeLocaleSignal(),{type:'region'}),rows:{value:string;name:string}[]=[];if(props.kind!=='region')return rows;for(let a=65;a<=90;a++)for(let b=65;b<=90;b++){const value=String.fromCharCode(a,b),name=names.of(value);if(name&&name!==value&&value!=='ZZ')rows.push({value,name});}return rows.sort((a,b)=>a.name.localeCompare(b.name));});
 const normalized=()=>props.kind==='region'?(draft().trim().toLowerCase()==='auto'?'auto':draft().trim().toUpperCase()):draft();
 const valid=()=>{const value=normalized();return props.kind==='region'?value==='auto'||/^[A-Z]{2}$/.test(value)&&regions().some(row=>row.value===value):validRegionalFormat(value,props.kind==='dateFormat'?'date':'time');};
 async function save(event:SubmitEvent):Promise<void>{event.preventDefault();if(!valid()){setError(t('regional-invalid'));return;}setError('');await setting.set(normalized());}
 async function reset():Promise<void>{if(await setting.reset()){setDraft(setting.value());setError('');}}
 const id=`regional-${props.kind}`;
 return <form onSubmit={event=>void save(event)}><SettingRow stacked controlId={id} label={t(`regional-${props.kind}`)} help={t(`regional-${props.kind}-help`)} handle={setting} onReset={()=>void reset()} resetDisabled={disabled()}>{describedBy=><>
  <input id={id} class={css.input} value={draft()} disabled={disabled()} maxlength={120} list={`${id}-options`} aria-describedby={describedBy()} onInput={event=>setDraft(event.currentTarget.value)}/>
  <datalist id={`${id}-options`}><Show when={props.kind==='region'} fallback={<For each={props.kind==='dateFormat'?DATE_FORMATS:TIME_FORMATS}>{value=><option value={value}>{value==='auto'?t('regional-auto'):preview({[props.kind]:value})}</option>}</For>}><option value="auto">{t('regional-auto')}</option><For each={regions()}>{row=><option value={row.value}>{row.name}</option>}</For></Show></datalist>
  <Show when={valid()}><output class={css.meta} aria-label={t('regional-preview')}>{preview({[props.kind]:normalized()})}</output></Show>
  <Show when={error()||setting.error()}><p role="alert" class={css.error}>{error()||setting.error()}</p></Show>
  <button type="submit" class={css.button} disabled={disabled()||draft()===setting.value()}>{t('common-save')}</button>
 </>}</SettingRow><LeaveDialog/></form>;
}
export function RegionalFormats(props:{navigation:SettingsNavigation}):JSX.Element {
 void loadCatalog('regional');
 const clock=useSetting<string>('time.hourCycle','auto'),week=useSetting<string>('time.firstDayOfWeek','auto');
 return <SettingsCard id="regional-card" title={t('regional-title')} description={t('regional-help')}>
  <For each={['region','dateFormat','timeFormat'] as const}>{kind=><FormatField kind={kind} navigation={props.navigation}/>}</For>
  <SelectRow id="regional-hourCycle" label={t('regional-hourCycle')} help={t('regional-hourCycle-help')} handle={clock} options={['auto','12','24'].map(value=>({value,label:t(`regional-clock-${value}`)}))} accept={(value):value is string=>['auto','12','24'].includes(value)}/>
  <SelectRow id="regional-firstDayOfWeek" label={t('regional-firstDayOfWeek')} help={t('regional-firstDayOfWeek-help')} handle={week} options={['auto','1','2','3','4','5','6','0'].map(value=>({value,label:value==='auto'?t('regional-auto'):new Intl.DateTimeFormat(regionalLocale(activeLocaleSignal()),{weekday:'long',timeZone:'UTC'}).format(new Date(Date.UTC(2026,10,22+Number(value))))}))} accept={(value):value is string=>value==='auto'||/^[0-6]$/.test(value)}/>
  <p class={css.meta}><span>{t('regional-preview')}: </span><output>{preview()}</output></p>
 </SettingsCard>;
}
export const REGIONAL_CARD:CardSpec={id:'regional-card',section:'time',title:'regional-title',help:'regional-help',rows:['region','dateFormat','timeFormat','hourCycle','firstDayOfWeek'].map(key=>({id:`regional-${key}`,label:`regional-${key}`,help:`regional-${key}-help`}))};
