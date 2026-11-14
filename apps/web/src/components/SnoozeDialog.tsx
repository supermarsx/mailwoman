import {createEffect,createSignal,createUniqueId,For,onCleanup,Show,type JSX} from 'solid-js';
import {Portal} from 'solid-js/web';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {formatDateWith,loadCatalog,t} from '../i18n';
import {wallTimeCandidates} from '../i18n/timezones.ts';
import {snoozeWallInput,snoozeZone} from '../state/snooze.ts';
import {createFocusTrap} from './a11y';
import * as css from '../screens/Settings/styles.css.ts';
import * as layout from '../screens/Settings/editor.css.ts';
interface Request {kind:'snooze'|'followUp';owner:PreferenceAccount;save:(until:string)=>Promise<void>;trigger:HTMLElement|null}
const [request,setRequest]=createSignal<Request>();
export function requestCustomSnooze(save:(until:string)=>Promise<void>,kind:'snooze'|'followUp'='snooze'):void{
 const owner=preferenceAccount();if(!owner)return;
 setRequest({kind,owner,save,trigger:document.activeElement instanceof HTMLElement?document.activeElement:null});
}
export function requestCustomFollowUp(save:(until:string)=>Promise<void>):void{requestCustomSnooze(save,'followUp');}
export function SnoozeDialog():JSX.Element{
 onCleanup(observePreferenceAccount(()=>{if(request()&&!preferenceAccountIsCurrent(request()!.owner))setRequest(undefined);}));
 return <Show when={request()} keyed>{value=><Picker request={value}/>}</Show>;
}
function Picker(props:{request:Request}):JSX.Element{
 void loadCatalog('snooze');
 const uid=createUniqueId(),zone=snoozeZone();
 const [element,setElement]=createSignal<HTMLDivElement>(),[value,setValue]=createSignal(snoozeWallInput(new Date(Date.now()+2*3600000),zone));
 const [choice,setChoice]=createSignal(''),[busy,setBusy]=createSignal(false),[error,setError]=createSignal('');
 const candidates=()=>wallTimeCandidates(value(),zone);
 const close=():void=>{if(busy())return;setRequest(undefined);if(props.request.trigger?.isConnected)props.request.trigger.focus();};
 createFocusTrap(element,{onEscape:close});
 createEffect(()=>{if(!preferenceAccountIsCurrent(props.request.owner))setRequest(undefined);});
 async function submit(event:SubmitEvent):Promise<void>{
  event.preventDefault();if(busy()||!preferenceAccountIsCurrent(props.request.owner))return;
  const matches=candidates(),at=matches.length===1?matches[0]:matches.find(item=>item.toISOString()===choice());
  if(!at||at.getTime()<=Date.now()){setError(t(matches.length>1&&!choice()?'snooze-ambiguous':'snooze-invalid-date'));return;}
  setBusy(true);setError('');
  try{await props.request.save(at.toISOString());if(preferenceAccountIsCurrent(props.request.owner)){setBusy(false);close();}}
  catch{if(preferenceAccountIsCurrent(props.request.owner))setError(t('common-error-retry-help'));}
  finally{setBusy(false);}
 }
 return <Portal><div class={layout.scrim}/><div class={layout.dialog} ref={setElement} role="dialog" aria-modal="true" aria-labelledby={`${uid}-title`} tabindex="-1">
  <h2 id={`${uid}-title`} class={css.heading}>{t('snooze-custom')}</h2>
  <form class={layout.fields} onSubmit={event=>void submit(event)}>
   <label class={css.field}>{t('snooze-custom')}<input type="datetime-local" class={css.input} required value={value()} disabled={busy()} onInput={event=>{setValue(event.currentTarget.value);setChoice('');setError('');}}/></label>
   <p class={css.meta}>{t('snooze-zone',{zone})}</p>
   <Show when={candidates().length>1}><label class={css.field}>{t('snooze-ambiguous')}<select class={css.select} value={choice()} disabled={busy()} onChange={event=>setChoice(event.currentTarget.value)}><option value="">{t('snooze-choose')}</option><For each={candidates()}>{at=><option value={at.toISOString()}>{formatDateWith(at,{timeZone:zone,dateStyle:'medium',timeStyle:'long'})}</option>}</For></select></label></Show>
   <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
   <div class={css.actions}><button type="submit" class={css.button} disabled={busy()}>{t(props.request.kind==='followUp'?'mail-flag':'mail-snooze')}</button><button type="button" class={css.ghost} disabled={busy()} onClick={close}>{t('common-cancel')}</button></div>
  </form>
 </div></Portal>;
}
