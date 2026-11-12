import {For,Show,createResource,createSignal,useContext,type JSX} from 'solid-js';
import {loadCatalog,t,formatDateTime} from '../../../i18n';
import {adminRequest} from '../shell/adminApi.ts';
import {ConfirmDialog} from '../shell/ConfirmDialog.tsx';
import {ShellContext} from '../shell/context.ts';
import {describeAdminError} from '../shell/Feedback.tsx';
import {InlineSettingEditor} from '../Settings/InlineSettingEditor.tsx';
import {useSettingRows} from './useSettingRows.ts';
import * as css from '../admin.css.ts';
import * as ccss from './compliance.css.ts';

export const MAIL_KEYS=['compliance.mail.enabled','compliance.mail.host','compliance.mail.port','compliance.mail.security','compliance.mail.from','compliance.mail.username','compliance.mail.allowPrivate','compliance.mail.requireProxy','retention.complianceMail.days','dsr.notify.emailAdmins','dsr.notify.operatorAddresses','dsr.notify.emailRequester','breach.notify.email','breach.notify.operatorAddresses','breach.subjects.email'] as const;
interface Delivery {id:string;event:string;status:string;at:number;attempts:number;error:string}
interface Status {items:Delivery[];passwordConfigured:boolean}
export default function NotificationsTab():JSX.Element{
 void loadCatalog('admin-governance');
 const shell=useContext(ShellContext),settings=useSettingRows(MAIL_KEYS);
 const [status,{refetch}]=createResource(async()=>(await adminRequest<Status>('/compliance/mail')).data);
 const [review,setReview]=createSignal<{row:Delivery;action:'retry'|'dismiss'}>();
 const [busy,setBusy]=createSignal(false),[failure,setFailure]=createSignal<unknown>(),[queued,setQueued]=createSignal(false);
 async function test():Promise<void>{if(busy())return;setBusy(true);setFailure(undefined);try{await adminRequest('/compliance/mail/test',{method:'POST',body:{}});setQueued(true);await refetch();}catch(error){setFailure(error);}finally{setBusy(false);}}
 return <div class={css.section} data-testid="compliance-notifications">
  <p class={css.note}>{t('governance-mail-help')}</p>
  <p>{t(status()?.passwordConfigured?'governance-mail-secret-ready':'governance-mail-secret-help')}</p>
  <For each={settings.rows()}>{row=><InlineSettingEditor settingKey={row.key} setting={row} label={t(`governance-mail-${row.key}`)} onChanged={()=>settings.reload()}/>}</For>
  <div class={ccss.row}><button class="btn" disabled={busy()||!shell?.can.can('settings.write')} onClick={()=>void test()}>{t('governance-mail-test')}</button><button class="btn btn--ghost" onClick={()=>void refetch()}>{t('governance-mail-refresh')}</button></div>
  <Show when={queued()}><p role="status">{t('governance-mail-queued')}</p></Show>
  <Show when={failure()||status.error}><p role="alert">{describeAdminError(failure()||status.error)}</p></Show>
  <div class={css.tableWrap}><table class={css.table}><thead><tr><th>{t('governance-mail-event')}</th><th>{t('governance-field-status')}</th><th>{t('governance-mail-time')}</th><th>{t('governance-mail-attempts')}</th><th>{t('governance-mail-result')}</th><th>{t('compliance-records-actions')}</th></tr></thead><tbody><For each={status()?.items??[]}>{row=><tr><td>{row.event}</td><td>{t(`governance-option-${row.status}`)}</td><td>{formatDateTime(row.at*1000)}</td><td>{row.attempts}</td><td>{row.error}</td><td><Show when={['failed','uncertain'].includes(row.status)&&shell?.can.can('settings.write')}><button class="btn btn--ghost" onClick={()=>setReview({row,action:'retry'})}>{t('common-retry')}</button><button class="btn btn--ghost" onClick={()=>setReview({row,action:'dismiss'})}>{t('governance-mail-dismiss')}</button></Show></td></tr>}</For></tbody></table></div>
  <Show when={review()} keyed>{value=><ConfirmDialog open title={t(value.action==='retry'?'common-retry':'governance-mail-dismiss')} body={t('governance-mail-retry-warning')} confirmLabel={t(value.action==='retry'?'common-retry':'governance-mail-dismiss')} danger={value.action==='retry'} onCancel={()=>setReview(undefined)} onConfirm={async()=>{await adminRequest(`/compliance/mail/${encodeURIComponent(value.row.id)}/resolve`,{method:'POST',body:{action:value.action,acceptDuplicateRisk:true}});setReview(undefined);await refetch();}}/>}</Show>
 </div>;
}
