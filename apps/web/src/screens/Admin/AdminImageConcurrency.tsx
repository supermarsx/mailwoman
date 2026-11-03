import {createSignal,For,onCleanup,Show,type JSX} from 'solid-js';
import type {AdminSetting} from '../../api/admin/settings.ts';
import {IconSelect} from '../../components/IconSelect.tsx';
import type {IconName} from '../../components/Icon.tsx';
import {formatBytes,formatNumber,t} from '../../i18n/index.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import * as layout from './privacyLayout.css.ts';

export const IMAGE_CONCURRENCY_KEY='privacy.imageConcurrencyMode';
export const IMAGE_CONCURRENCY_MODES=['auto','conservative','balanced','throughput','manual'] as const;
export type ImageConcurrencyMode=typeof IMAGE_CONCURRENCY_MODES[number];
export interface ImageConcurrencyLimits {perMessage:number;perAccount:number;global:number;pipeline:number;decoders:number}
export interface ImageConcurrencyRuntime {
 mode:string;adaptive:boolean;cpuCores:number;memoryLimitBytes:number|null;memoryAvailableBytes:number|null;
 ceilings:ImageConcurrencyLimits;effective:ImageConcurrencyLimits;
 active:{network:number;pipeline:number;decoders:number;queued:number};
}
const modeOf=(value:unknown):ImageConcurrencyMode|undefined=>IMAGE_CONCURRENCY_MODES.find(mode=>mode===value);
const icons:Record<ImageConcurrencyMode,IconName>={auto:'assistant',conservative:'shield',balanced:'preferences',throughput:'bolt',manual:'wrench'};
const count=(value:unknown):string=>typeof value==='number'&&Number.isSafeInteger(value)&&value>=0?formatNumber(value):t('privacy-concurrency-unavailable');
const cpu=(value:unknown):string=>typeof value==='number'&&Number.isFinite(value)&&value>0?formatNumber(value,{maximumFractionDigits:3}):t('privacy-concurrency-unavailable');
const memory=(value:unknown):string=>typeof value==='number'&&Number.isFinite(value)&&value>=0?formatBytes(value):t('privacy-concurrency-unavailable');

/** A single ETag-protected enum write chooses a preset; numeric ceilings stay intact. */
export function AdminImageConcurrency(props:{setting?:AdminSetting|undefined;runtime?:ImageConcurrencyRuntime|undefined;canWrite:boolean;save?:(mode:ImageConcurrencyMode,etag:string)=>Promise<AdminSetting>;onSaved:(row:AdminSetting)=>void;onReload?:()=>void}):JSX.Element{
 const [choice,setChoice]=createSignal<ImageConcurrencyMode>();const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [saved,setSaved]=createSignal(false);let disposed=false;onCleanup(()=>{disposed=true;});
 const current=()=>modeOf(props.setting?.effective.value);
 const selected=()=>choice()??current();
 const pinned=()=>props.setting?.envState?.set===true&&props.setting.envState.precedence==='pin';
 const disabled=()=>!props.canWrite||busy()||!current()||pinned()||props.setting?.dedicatedRoute!=null||!props.save;
 async function apply():Promise<void>{const mode=selected(),row=props.setting;if(disabled()||!mode||!row||mode===current())return;setBusy(true);setError('');setSaved(false);try{const next=await props.save!(mode,row.etag);if(disposed)return;props.onSaved(next);setChoice(undefined);setSaved(true);}catch(cause){if(!disposed){setError(describeAdminError(cause));props.onReload?.();}}finally{if(!disposed)setBusy(false);}}
 const fields:readonly (keyof ImageConcurrencyLimits)[]=['perMessage','perAccount','global','pipeline','decoders'];
 return <div class={layout.concurrency} data-testid="image-concurrency-profile">
  <Show when={current()}>
   <div class={layout.profileControls}>
    <IconSelect label={t('admin-settings-key-privacy-image-concurrency-mode')} value={selected()!} showLabel disabled={disabled()} options={IMAGE_CONCURRENCY_MODES.map(mode=>({value:mode,label:t(`privacy-concurrency-mode-${mode}`),icon:icons[mode]}))} onChange={value=>{if(!disabled()){setChoice(modeOf(value));setSaved(false);setError('');}}}/>
    <button type="button" class="btn" disabled={disabled()||selected()===current()} aria-busy={busy()} onClick={()=>void apply()}>{t('privacy-concurrency-apply')}</button>
   </div>
   <p>{t('admin-settings-help-privacy-image-concurrency-mode')}</p>
  </Show>
  <Feedback kind="error" as="toast" message={error()}/><Feedback kind="success" message={saved()?t('privacy-concurrency-saved'):''}/>
  <Show when={props.runtime} fallback={<p>{t('privacy-concurrency-unavailable')}</p>}>{runtime=><div data-testid="image-concurrency-runtime"><p>{t('privacy-concurrency-sizing',{cpu:cpu(runtime().cpuCores),limit:memory(runtime().memoryLimitBytes),available:memory(runtime().memoryAvailableBytes)})}</p><p>{t('privacy-concurrency-running',{mode:modeOf(runtime().mode)?t(`privacy-concurrency-mode-${runtime().mode}`):t('privacy-concurrency-unavailable'),state:typeof runtime().adaptive==='boolean'?t(runtime().adaptive?'privacy-on':'privacy-off'):t('privacy-concurrency-unavailable')})}</p><dl class={layout.capacities}><For each={fields}>{field=><div><dt>{t(`privacy-concurrency-cap-${field}`)}</dt><dd>{t('privacy-concurrency-effective-ceiling',{effective:count(runtime().effective?.[field]),ceiling:count(runtime().ceilings?.[field])})}</dd></div>}</For></dl><p>{t('privacy-concurrency-message-bound')}</p><p>{t('privacy-concurrency-active',{network:count(runtime().active?.network),pipeline:count(runtime().active?.pipeline),decoders:count(runtime().active?.decoders),queued:count(runtime().active?.queued)})}</p></div>}</Show>
 </div>;
}
