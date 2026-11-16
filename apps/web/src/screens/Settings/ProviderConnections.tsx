import {createResource,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {providerRequest,providerHttpsUrl,rememberProviderFlow,forgetProviderFlow,pendingProviderFlow,ProviderOauthError,type ProviderStatus,type ProviderFlow,type ProviderBridge} from '../../api/providerOauth.ts';
import {loadCatalog,t} from '../../i18n';
import {Icon} from '../../components/Icon.tsx';
import {toast} from '../../components/toast/toast.ts';
import * as css from './styles.css.ts';
const providers:readonly ProviderBridge[]=['bridge-graph','bridge-gmail'];
const label=(id:ProviderBridge)=>id==='bridge-graph'?'Microsoft Graph':'Gmail API';
const failure=(cause:unknown)=>t(cause instanceof ProviderOauthError&&cause.code==='provider_migration_required'?'provider-oauth-migration':'provider-oauth-failed');
export function ProviderConnections():JSX.Element{
 const controller=new AbortController();let timer:ReturnType<typeof setTimeout>|undefined;
 onMount(()=>void loadCatalog('linked-accounts'));onCleanup(()=>{controller.abort();clearTimeout(timer);});
 const [status,{refetch}]=createResource(()=>providerRequest<ProviderStatus>('','GET',undefined,controller.signal));
 const [bridge,setBridge]=createSignal<ProviderBridge>('bridge-graph'),[clientId,setClientId]=createSignal(''),[tenant,setTenant]=createSignal(''),[busy,setBusy]=createSignal(false),[flow,setFlow]=createSignal<ProviderFlow>();
 const [connected,setConnected]=createSignal(false);
 const polling=new Set<string>();
 const current=(pending:ProviderFlow)=>flow()?.flowId===pending.flowId&&!controller.signal.aborted;
 function schedulePoll(pending:ProviderFlow,interval=pending.interval??5):void{clearTimeout(timer);timer=setTimeout(()=>void poll(pending),Math.max(5,Math.min(60,interval))*1000);}
 const configuredId=()=>status()?.bridgeId===bridge()?status()?.clientId??'':'';
 const effectiveTenant=()=>bridge()==='bridge-graph'?(tenant().trim()||(status()?.bridgeId===bridge()?status()?.tenant??'':'')):'';
 async function poll(pending:ProviderFlow):Promise<void>{
  if(!current(pending)||polling.has(pending.flowId))return;
  polling.add(pending.flowId);
  try{const result=await providerRequest<{status:string;interval?:number}>('/complete','POST',{flowId:pending.flowId},controller.signal);
   if(!current(pending))return;
   if(result.status==='connected'){forgetProviderFlow();setConnected(true);setFlow(undefined);void refetch();toast.success(t('provider-oauth-connected'));return;}
   if(result.status!=='pending'&&result.status!=='slowDown')throw new Error('Invalid provider status');
   schedulePoll(pending,result.interval??pending.interval??5);
  }catch(cause){if(current(pending)){
   if(cause instanceof ProviderOauthError&&['provider_busy','provider_committing','provider_activation_failed'].includes(cause.code)){schedulePoll(pending);if(cause.code==='provider_activation_failed')toast.error(t('provider-oauth-failed'));}
   else{setFlow(undefined);forgetProviderFlow();toast.error(failure(cause));}
  }}finally{polling.delete(pending.flowId);}
 }
 async function start(kind:'authorizationCode'|'deviceCode'):Promise<void>{
  if(busy())return;setBusy(true);setConnected(false);
  try{
   const pending=await providerRequest<ProviderFlow>('/start','POST',{bridgeId:bridge(),clientId:clientId().trim()||configuredId(),...(effectiveTenant()?{tenant:effectiveTenant()}:{}),flow:kind},controller.signal);
   rememberProviderFlow(pending);
   if(kind==='authorizationCode'){const url=providerHttpsUrl(pending.authorizationUrl);if(!url)throw new Error('Invalid authorization URL');location.assign(url);}
   else{if(!providerHttpsUrl(pending.verificationUri)||!pending.userCode)throw new Error('Invalid verification response');setFlow(pending);schedulePoll(pending);}
  }catch(cause){if(!controller.signal.aborted){forgetProviderFlow();toast.error(failure(cause));}}finally{setBusy(false);}
 }
 async function cancel():Promise<void>{
  const pending=flow();if(!pending||busy())return;clearTimeout(timer);setBusy(true);
  try{await providerRequest(`/${encodeURIComponent(pending.flowId)}`,'DELETE',undefined,controller.signal);if(current(pending)){clearTimeout(timer);setFlow(undefined);forgetProviderFlow();}}
  catch(cause){if(current(pending)){
   if(cause instanceof ProviderOauthError&&cause.code==='provider_committing')toast.info(t('common-loading'));
   else toast.error(t('provider-oauth-failed'));
   // Once credentials are being committed cancellation is explicitly refused.
   // Keep correlation and let the in-flight completion (or its retry) finish.
   if(!polling.has(pending.flowId))schedulePoll(pending);
  }}finally{if(!controller.signal.aborted)setBusy(false);}
 }
 return <section class={css.section} aria-label={t('provider-oauth-title')}>
  <h3 class={css.heading}>{t('provider-oauth-title')}</h3><p class={css.meta}>{t('provider-oauth-help')}</p>
  <Show when={!status.error} fallback={<><p class={css.meta}>{t('provider-oauth-failed')}</p><button class={css.ghost} type="button" onClick={()=>void refetch()}>{t('common-retry')}</button></>}>
   <Show when={status.loading}><p role="status">{t('common-loading')}</p></Show>
   <Show when={connected()||status()?.connected}><p role="status">{t('provider-oauth-connected')} <Show when={status()?.bridgeId}>{id=>label(id())}</Show></p></Show>
   <Show when={!flow()} fallback={<div class={css.field}><p>{t('provider-oauth-device-help')}</p><code>{flow()?.userCode}</code><a class={css.button} href={providerHttpsUrl(flow()?.verificationUri)} target="_blank" rel="noopener noreferrer">{t('common-open')}</a><div class={css.actions}><button class={css.ghost} type="button" disabled={busy()} onClick={()=>void cancel()}>{t('common-cancel')}</button></div></div>}>
    <label class={css.field}>{t('provider-oauth-provider')}<select class={css.select} value={bridge()} disabled={busy()} onChange={event=>{const value=event.currentTarget.value as ProviderBridge;if(providers.includes(value)){setBridge(value);setClientId('');setTenant('');}}}><For each={providers}>{id=><option value={id}>{label(id)}</option>}</For></select></label>
    <label class={css.field}>{t('provider-oauth-client-id')}<input class={css.input} maxLength={256} value={clientId()} placeholder={configuredId()} disabled={busy()} onInput={event=>setClientId(event.currentTarget.value)}/></label>
    <Show when={bridge()==='bridge-graph'}><label class={css.field}>{t('provider-oauth-tenant')}<input class={css.input} maxLength={128} value={tenant()} placeholder={status()?.tenant??'common'} disabled={busy()} onInput={event=>setTenant(event.currentTarget.value)}/></label></Show>
    <div class={css.actions}><button class={css.button} type="button" disabled={busy()||status.loading||!(clientId().trim()||configuredId())} onClick={()=>void start('authorizationCode')}><Icon name="link"/>{t('provider-oauth-connect')}</button><Show when={bridge()==='bridge-graph'}><button class={css.ghost} type="button" disabled={busy()||status.loading||!(clientId().trim()||configuredId())} onClick={()=>void start('deviceCode')}>{t('provider-oauth-device')}</button></Show></div>
   </Show>
  </Show>
 </section>;
}
export function ProviderOauthCallback():JSX.Element{
 const controller=new AbortController(),params=new URLSearchParams(location.search),id=pendingProviderFlow();
 const [state,setState]=createSignal<'loading'|'done'|'error'>('loading');
 const [retryable,setRetryable]=createSignal(false),[error,setError]=createSignal('');
 const clean=new URL(location.href);for(const key of ['code','state','error','error_description','provider-oauth-callback','session_state'])clean.searchParams.delete(key);history.replaceState(history.state,'',clean.pathname+clean.search+clean.hash);
 onCleanup(()=>controller.abort());
 async function complete():Promise<void>{
  if(!id||controller.signal.aborted)return;setState('loading');setRetryable(false);
  try{const result=await providerRequest<{status:string}>('/complete','POST',{flowId:id,code:params.get('code'),state:params.get('state')},controller.signal);
   if(controller.signal.aborted)return;
   if(result.status!=='connected')throw new ProviderOauthError('provider_authorization_failed');
   forgetProviderFlow();setState('done');
  }catch(cause){if(!controller.signal.aborted){
   const retry=!(cause instanceof ProviderOauthError)||['provider_activation_failed','provider_busy','provider_committing','provider_unavailable'].includes(cause.code);
   if(!retry)forgetProviderFlow();setRetryable(retry);setError(failure(cause));setState('error');
  }}
 }
 onMount(()=>{void loadCatalog('linked-accounts');
  if(!id||params.has('error')||!params.get('code')||!params.get('state')){forgetProviderFlow();setState('error');return;}
  void complete();
 });
 return <section class={css.section} aria-label={t('provider-oauth-title')}><h2 class={css.heading}>{t('provider-oauth-title')}</h2><p role="status">{state()==='error'?(error()||t('provider-oauth-failed')):t(state()==='loading'?'common-loading':'provider-oauth-connected')}</p><Show when={state()!=='loading'}><div class={css.actions}><Show when={retryable()}><button class={css.button} type="button" onClick={()=>void complete()}>{t('common-retry')}</button></Show><button class={css.ghost} type="button" onClick={()=>{forgetProviderFlow();location.reload();}}>{t('common-close')}</button></div></Show></section>;
}
