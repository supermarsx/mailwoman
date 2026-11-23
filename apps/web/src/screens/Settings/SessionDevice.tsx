import {Show,onMount,type JSX} from 'solid-js';
import {Icon} from '../../components/Icon.tsx';
import {loadCatalog,t} from '../../i18n';
import {formatDateTime,formatLocale} from '../../i18n/format.ts';
import {describeSessionDevice,sessionDate} from './sessionDevice.ts';
import * as css from './sessionDevice.css.ts';

export function SessionDevice(props:{userAgent?:string|null|undefined;deviceLabel?:string|null|undefined;badge?:JSX.Element}):JSX.Element {
  onMount(()=>{void loadCatalog('security');});
  const agent=()=>props.userAgent||(/^(?:Mozilla\/|Opera\/)/.test(props.deviceLabel??'')?props.deviceLabel:'');
  const info=()=>describeSessionDevice(agent());
  const label=()=>props.deviceLabel&&!/^(?:Mozilla\/|Opera\/)/.test(props.deviceLabel)?props.deviceLabel:'';
  return <div class={css.device}>
    <div class={css.line}><Icon name={['phone','tablet'].includes(info().device)?'smartphone':info().device==='desktop'?'laptop':'info'}/><strong><bdi>{label()||t(`session-evidence-device-${info().device}`)}</bdi></strong>{props.badge}</div>
    <div class={css.metadata}>
      <Show when={info().browser}><span class={css.line}><Icon name="globe"/><span><bdi>{info().browser}</bdi></span></span></Show>
      <Show when={info().os}><span class={css.line}><Icon name="chip"/><span><bdi>{info().os}</bdi></span></span></Show>
    </div>
    <Show when={agent()}><details class={css.technical}><summary>{t('session-evidence-technical')}</summary><p class={css.agent}><bdi>{agent()}</bdi></p></details></Show>
  </div>;
}

export function sessionTimestamp(value?:string|null):string {
  const date=sessionDate(value);
  return date?formatDateTime(date,{dateStyle:'medium',timeStyle:'short'}):'—';
}

export function sessionCountry(code:string):string {
  if(!/^[A-Za-z]{2}$/.test(code))return code;
  try{return new Intl.DisplayNames([formatLocale()],{type:'region'}).of(code.toUpperCase())??code;}catch{return code;}
}
