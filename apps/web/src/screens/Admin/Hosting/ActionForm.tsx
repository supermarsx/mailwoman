import { Show, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import type { HostingConnection, HostingOperation } from '../../../api/admin/hosting.ts';
import { Feedback } from '../shell/Feedback.tsx';
import { useDirtyGuard } from '../shell/useDirtyGuard.ts';
import { describeHostingError } from './errors.ts';
import * as css from '../admin.css.ts';
import * as ux from '../shell/ux.css.ts';
export type HostingFormKind='domain'|'mailbox'|'quota'|'password'|'alias';
export function HostingActionForm(props:{kind:HostingFormKind;connection:HostingConnection;domain?:string;account?:string;quotaMiB?:number|null;canWrite:boolean;submit:(operation:HostingOperation)=>Promise<void>;back:()=>void}):JSX.Element {
 const [domain,setDomain]=createSignal(props.domain??'');const [account,setAccount]=createSignal(props.account??'');const [password,setPassword]=createSignal('');const [parent,setParent]=createSignal('');const [owner,setOwner]=createSignal('');const [displayName,setDisplayName]=createSignal('');const [quota,setQuota]=createSignal(props.quotaMiB?.toString()??'');const [alias,setAlias]=createSignal('');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal<string>();const [saved,setSaved]=createSignal(false);
 const snapshot=():string=>JSON.stringify([domain(),account(),password(),parent(),owner(),displayName(),quota(),alias()]);const baseline=snapshot();useDirtyGuard(()=>!saved()&&snapshot()!==baseline);
 const title=():string=>t(`admin-hosting-form-${props.kind}`);
 async function submit(event:SubmitEvent):Promise<void>{
  event.preventDefault();if(!props.canWrite||busy())return;setBusy(true);setError(undefined);
  try{
   const amount=quota()===''?undefined:Number(quota());if(amount!==undefined&&(!Number.isSafeInteger(amount)||amount<0||amount>4294967295))throw new Error(t('admin-hosting-invalid-quota'));
   let operation:HostingOperation;
   switch(props.kind){
    case 'domain':operation={action:'createDomain',domain:domain(),...(props.connection.provider==='virtualmin'?(parent()?{parent:parent()}:{owner:owner(),password:password()}):{})};break;
    case 'mailbox':operation={action:'createMailbox',domain:domain(),account:account(),password:password(),...(amount!==undefined?{quotaMiB:amount}:{}),...(props.connection.provider==='virtualmin'&&displayName()?{displayName:displayName()}:{})};break;
    case 'quota':if(amount===undefined)throw new Error(t('admin-hosting-invalid-quota'));operation={action:'setMailboxQuota',domain:domain(),account:account(),quotaMiB:amount};break;
    case 'password':operation={action:'resetMailboxPassword',domain:domain(),account:account(),password:password()};break;
    case 'alias':operation={action:'addAlias',domain:domain(),account:account(),alias:alias()};break;
   }
   setSaved(true);await props.submit(operation);setPassword('');
  }catch(cause){setSaved(false);setError(describeHostingError(cause));}finally{setBusy(false);}
 }
 return <section class={css.section}><div class={ux.pageActions}><button class="btn" type="button" disabled={busy()} onClick={props.back}>{t('admin-hosting-back-inventory')}</button></div><h2>{title()}</h2><p class={css.note}>{props.connection.name}{props.domain?` · ${props.domain}`:''}{props.account?` · ${props.account}`:''}</p><Feedback kind="error" message={error()}/>
 <Show when={props.kind==='domain'&&props.connection.provider==='virtualmin'}><p class={css.note}>{t('admin-hosting-virtualmin-impact')}</p></Show>
 <Show when={props.connection.provider==='virtualmin'&&props.kind!=='domain'}><p class={css.note}>{t('admin-hosting-virtualmin-account-impact')}</p></Show><form onSubmit={(event)=>void submit(event)} aria-label={title()}><fieldset disabled={!props.canWrite||busy()}><legend>{t('admin-hosting-details')}</legend><div class={css.grid}>
 <Show when={props.kind==='domain'}><label class="field">{t('admin-hosting-domain')}<input required value={domain()} onInput={(e)=>setDomain(e.currentTarget.value)} /></label><Show when={props.connection.provider==='virtualmin'}><label class="field">{t('admin-hosting-parent')}<input value={parent()} onInput={(e)=>setParent(e.currentTarget.value)}/></label><Show when={!parent()}><label class="field">{t('admin-hosting-domain-owner')}<input required value={owner()} onInput={(e)=>setOwner(e.currentTarget.value)}/></label></Show></Show></Show>
 <Show when={props.kind==='mailbox'}><label class="field">{t('admin-hosting-account')}<input required value={account()} onInput={(e)=>setAccount(e.currentTarget.value)}/></label><Show when={props.connection.provider==='virtualmin'}><label class="field">{t('admin-hosting-display-name')}<input value={displayName()} onInput={(e)=>setDisplayName(e.currentTarget.value)}/></label></Show></Show>
 <Show when={props.kind==='mailbox'||props.kind==='password'||(props.kind==='domain'&&props.connection.provider==='virtualmin'&&!parent())}><label class="field">{t('admin-hosting-new-password')}<input type="password" autocomplete="new-password" required value={password()} onInput={(e)=>setPassword(e.currentTarget.value)}/></label></Show>
 <Show when={props.kind==='mailbox'||props.kind==='quota'}><label class="field">{t('admin-hosting-quota')}<input type="number" min={0} max={4294967295} step={1} required={props.kind==='quota'} value={quota()} onInput={(e)=>setQuota(e.currentTarget.value)}/></label></Show>
 <Show when={props.kind==='alias'}><label class="field">{t('admin-hosting-alias')}<input required value={alias()} onInput={(e)=>setAlias(e.currentTarget.value)}/></label><p class={css.note}>{t('admin-hosting-alias-help')}</p></Show>
 </div></fieldset><div class={ux.pageActions}><button class="btn btn--primary" type="submit" disabled={!props.canWrite||busy()}>{title()}</button></div></form></section>;
}
