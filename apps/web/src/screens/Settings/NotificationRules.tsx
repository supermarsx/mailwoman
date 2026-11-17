import {createSignal,For,Show,type JSX} from 'solid-js';
import {useEditorGuard} from './entityEditor.tsx';
import type {NotificationRule} from './types.ts';
import {toast} from '../../components/toast/toast.ts';
import {Icon} from '../../components/Icon.tsx';
import {t} from '../../i18n/index.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import * as css from './styles.css.ts';
import * as table from './linkedAccounts.css.ts';
import * as entity from './entityTable.css.ts';
export function NotificationRules(props:{rules:readonly NotificationRule[];save:(rules:readonly NotificationRule[])=>Promise<void>;back:()=>void}):JSX.Element{
 const [editing,setEditing]=createSignal<NotificationRule>(),[busy,setBusy]=createSignal(false);
 const open=(rule?:NotificationRule):void=>{setEditing(rule?{...rule}:{id:`r-${crypto.randomUUID()}`,label:'',match:'',action:'notify'});};
 async function remove(rule:NotificationRule):Promise<void>{if(busy())return;setBusy(true);try{await props.save(props.rules.filter(row=>row.id!==rule.id));toast.success(t('settings-alert-rule-deleted'));}catch{toast.error(t('settings-notif-error'),{action:{label:t('common-retry'),run:()=>remove(rule)}});}finally{setBusy(false);}}
 return <Show when={editing()} fallback={<SettingsCard id="notification-rules-page" title={t('settings-notif-rules-title')} description={t('settings-alert-rule-intro')}>
  <div class={table.header}><button type="button" class={css.ghost} onClick={props.back}><Icon name="back"/>{t('settings-alert-back')}</button><button type="button" class={`${css.button} ${table.addButton}`} disabled={busy()||props.rules.length>=64} onClick={()=>open()} data-testid="notif-add-rule"><Icon name="plus"/>{t('settings-notif-rule-add')}</button></div>
  <Show when={props.rules.length===0}><p class={css.meta}>{t('settings-alert-rule-empty')}</p></Show>
  <ul class={table.table} aria-label={t('settings-notif-rules-title')}><For each={props.rules}>{rule=><li class={entity.row}>
   <span class={entity.avatar}><Icon name={rule.action==='mute'?'close':'bell'}/></span><div class={table.who}><strong>{rule.label||rule.match}</strong><span class={table.address}>{rule.match} · {t(`settings-notif-rule-${rule.action}`)}</span></div>
   <div class={table.actions}><button type="button" class={css.ghost} disabled={busy()} aria-label={t('settings-edit')} onClick={()=>open(rule)}><Icon name="edit"/></button><button type="button" class={css.ghost} disabled={busy()} aria-label={t('settings-delete')} onClick={()=>void remove(rule)}><Icon name="delete"/></button></div>
  </li>}</For></ul>
 </SettingsCard>}>{rule=><RuleEditor initial={rule()} busy={busy} save={async value=>{setBusy(true);try{await props.save([...props.rules.filter(row=>row.id!==value.id),value]);setEditing(undefined);toast.success(t('settings-saved'));}finally{setBusy(false);}}} back={()=>setEditing(undefined)}/>}</Show>;
}
function RuleEditor(props:{initial:NotificationRule;busy:()=>boolean;save:(rule:NotificationRule)=>Promise<void>;back:()=>void}):JSX.Element{
 const [draft,setDraft]=createSignal({...props.initial});let saved=false;
 const guard=useEditorGuard(()=>!saved&&JSON.stringify(draft())!==JSON.stringify(props.initial),props.busy);
 async function save(event:SubmitEvent):Promise<void>{event.preventDefault();const value={...draft(),label:draft().label.trim(),match:draft().match.trim()};
  if(!value.match||value.match.length>256||value.label.length>128||/[\p{Cc}]/u.test(value.match+value.label)){toast.error(t('settings-alert-rule-invalid'));return;}
  try{saved=true;await props.save(value);}catch{saved=false;toast.error(t('settings-notif-error'));}
 }
 return <SettingsCard id="notification-rule-editor" title={t('settings-alert-rule-editor')} description={t('settings-alert-rule-intro')}><form noValidate class={css.stack} onSubmit={e=>void save(e)}>
  <label class={css.stack}>{t('settings-alert-rule-name')}<input class={css.input} maxlength={128} value={draft().label} onInput={e=>setDraft({...draft(),label:e.currentTarget.value})}/></label>
  <label class={css.stack}>{t('settings-notif-rule-match-placeholder')}<input class={css.input} required maxlength={256} value={draft().match} onInput={e=>setDraft({...draft(),match:e.currentTarget.value})}/></label>
  <label class={css.stack}>{t('settings-notif-rule-action-label')}<select class={css.select} value={draft().action} onChange={e=>setDraft({...draft(),action:e.currentTarget.value as NotificationRule['action']})}><option value="notify">{t('settings-notif-rule-notify')}</option><option value="mute">{t('settings-notif-rule-mute')}</option></select></label>
  <div class={css.actions}><button type="button" class={css.ghost} disabled={props.busy()} onClick={()=>void guard.leave(props.back)}>{t('settings-cancel')}</button><button type="submit" class={css.button} disabled={props.busy()}>{t('settings-save')}</button></div>
 </form>{guard.dialog()}</SettingsCard>;
}
