import {createSignal,Show,useContext} from 'solid-js';
import {ShellContext} from './shell/context.ts';
import {adminRequest} from './shell/adminApi.ts';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {describeAdminError} from './shell/Feedback.tsx';
import {Icon} from '../../components/Icon.tsx';
import {t} from '../../i18n/index.ts';
import type {UserSummary} from '../../state/slices/admin.ts';
import * as ux from './shell/ux.css.ts';
import * as css from './userLifecycle.css.ts';
interface Lifecycle {status:'active'|'disabled'|'deleting'|'deleted';directoryAddress:string;deletedAt:number|null;canRestore:boolean}
export function UserAccessActions(props:{user:UserSummary;onChanged:()=>Promise<void>;configureHref?:string}){
 const shell=useContext(ShellContext);const [pending,setPending]=createSignal<{action:'block'|'unblock'|'delete'|'legacy';data:Lifecycle;etag:string}|null>(null);const [busy,setBusy]=createSignal(false),[error,setError]=createSignal('');
 const account=()=>props.user.loginAddress??(props.user.domain?`${props.user.username}@${props.user.domain}`:props.user.username);
 const endpoint=()=>`/users/${encodeURIComponent(props.user.accountId)}/lifecycle`;
 const deleted=()=>props.user.status==='deleted'||props.user.status==='deleting';
 const blocked=()=>props.user.status==='disabled';
 async function prepare(action:'block'|'unblock'|'delete'|'legacy'){
  if(busy())return;setBusy(true);setError('');try{const response=await adminRequest<Lifecycle>(action==='legacy'?`/users/${encodeURIComponent(props.user.accountId)}/legacy-block`:endpoint());if(!response.etag)throw new Error('Missing revision');setPending({action,data:response.data,etag:response.etag});}catch(error){setError(describeAdminError(error));}finally{setBusy(false);}
 }
 async function confirm(){const change=pending();if(!change)return;await adminRequest<Lifecycle>(change.action==='legacy'?`/users/${encodeURIComponent(props.user.accountId)}/legacy-block`:endpoint(),{method:change.action==='delete'||change.action==='legacy'?'DELETE':'PUT',ifMatch:change.etag,...(change.action==='delete'||change.action==='legacy'?{}:{body:{disabled:change.action==='block'}})});setPending(null);await props.onChanged();}
 return <div class={css.root}><div class={props.configureHref?css.actions:css.detailActions}>
  <Show when={props.configureHref}>{href=><a class={`btn ${ux.iconAction} ${css.iconLink}`} href={href()} aria-label={t('admin-users-configure-for',{account:account()})} data-tooltip={t('admin-users-configure')}><Icon name="settings"/></a>}</Show>
  <button type="button" class={props.configureHref?ux.iconAction:`btn ${css.textAction}`} disabled={busy()||deleted()||props.user.flags.disabled||!(shell?.can.can('users.flags')??true)} aria-label={t(blocked()?'admin-users-unblock-for':'admin-users-block-for',{account:account()})} data-tooltip={t(blocked()?'admin-users-unblock-help':'admin-users-block-help')} onClick={()=>void prepare(blocked()?'unblock':'block')}><Icon name={blocked()?'unlock':'lock'}/><Show when={!props.configureHref}><span>{t(blocked()?'admin-users-lifecycle-unblock-confirm':'admin-users-lifecycle-block-confirm')}</span></Show></button>
  <button type="button" class={props.configureHref?ux.iconAction:`btn ${css.textAction}`} disabled={busy()||deleted()||!(shell?.can.can('users.delete')??true)} aria-label={t('admin-users-delete-for',{account:account()})} data-tooltip={t('admin-users-delete-help')} onClick={()=>void prepare('delete')}><Icon name="delete"/><Show when={!props.configureHref}><span>{t('admin-users-lifecycle-delete-confirm')}</span></Show></button>
 <Show when={props.user.flags.disabled}><button type="button" class={props.configureHref?ux.iconAction:`btn ${css.textAction}`} disabled={busy()||deleted()||!(shell?.can.can('users.flags')??true)} aria-label={t('admin-users-legacy-clear')} data-tooltip={t('admin-users-legacy-block-help')} onClick={()=>void prepare('legacy')}><Icon name="unlock"/><Show when={!props.configureHref}><span>{t('admin-users-legacy-clear')}</span></Show></button></Show>
 </div><Show when={props.user.flags.disabled}><p>{t('admin-users-legacy-block-help')}</p></Show><Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
 <Show when={pending()}>{change=><ConfirmDialog open title={t(`admin-users-lifecycle-${change().action}-title`)} body={t(`admin-users-lifecycle-${change().action}-body`)} confirmLabel={t(`admin-users-lifecycle-${change().action}-confirm`)} danger={change().action!=='unblock'} {...(change().action==='delete'?{typedConfirmation:change().data.directoryAddress}:{})} onConfirm={confirm} onCancel={()=>setPending(null)}><p dir="auto">{account()}</p></ConfirmDialog>}</Show>
 </div>;
}
