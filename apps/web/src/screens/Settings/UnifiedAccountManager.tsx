import {createResource,createSignal,createEffect,on,For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {MailboxAvatarEditor} from './MailboxAvatarEditor.tsx';
import {workspaceMailboxes,workspaceFolders} from '../../api/workspaceMail.ts';
import {unifiedAccountKey,unifiedFolderKey,unifiedAvatar} from '../../state/unifiedAccountPreferences.ts';
import {requestUnifiedMailbox} from '../../state/unifiedMailbox.ts';
import {loadCatalog,t} from '../../i18n';
import {Icon} from '../../components/Icon.tsx';
import {LockNote} from './LockNote.tsx';
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
import * as css from './styles.css.ts';
import * as layout from './linkedSettingsInheritance.css.ts';
void loadCatalog('workspace-mail');
export function UnifiedAccountManager(props:{list:LinkedMailboxList;onChanged:()=>void}):JSX.Element{
 const enabled=useSetting<boolean>('mail.unifiedAccount',false),first=useSetting<boolean>('mail.unifiedAccount.openFirst',false),name=useSetting<string>('mail.unifiedAccount.name',''),avatar=useSetting<string>('mail.unifiedAccount.avatar','');
 const included=useSetting<string[]>('mail.unifiedAccount.accounts',['*']),folders=useSetting<string[]>('mail.unifiedAccount.folders',['*']);
 const allowed=useSetting<string[]>('mail.unifiedAccount.allowedAccounts',['*']),allowedFolders=useSetting<string[]>('mail.unifiedAccount.allowedFolders',['*']);
 createEffect(on(()=>JSON.stringify([enabled.value(),first.value(),name.value(),avatar.value(),included.value(),folders.value()]),()=>props.onChanged(),{defer:true}));
 const [editing,setEditing]=createSignal(false);
 const [choices]=createResource(editing,async open=>{if(!open)return [];const targets=await workspaceMailboxes(null);const controller=new AbortController();return Promise.all(targets.map(async target=>({target,folders:await workspaceFolders(target,controller.signal)})));});
 const label=()=>name.value().trim()||t('unified-mail-title');
 const has=(items:string[],key:string,role?:string|null)=>items.includes('*')||items.includes(key)||!!role&&items.includes(`role:${role}`);
 const canAccount=(key:string)=>has(allowed.value(),key);
 const toggle=(setting:typeof included,key:string,checked:boolean,all:string[])=>{const current=setting.value().includes('*')?all:setting.value();void setting.set(checked?[...new Set([...current,key])]:current.filter(value=>value!==key));};
 return <Show when={props.list.policy.enabled&&props.list.accounts.some(account=>account.status==='ready')}><section class={css.section} data-testid="unified-account-manager">
  <div class={css.actions}><MailboxAvatarEditor mailboxKey="@unified" label={label()} address="@unified" avatar={unifiedAvatar()} disabled={!avatar.entry()||avatar.locked()||avatar.saving()} onSave={value=>avatar.set(value?JSON.stringify(value):'')}/><div><h3 class={css.heading}>{label()}</h3><p class={css.meta}>{t('unified-mail-virtual-account')}</p></div>
  <button type="button" class={css.ghost} aria-pressed={first.value()} disabled={!first.entry()||first.locked()||first.saving()||!enabled.value()} onClick={()=>void first.set(!first.value()).then(saved=>{if(saved)props.onChanged();})}><Icon name={first.value()?'star':'starOutline'}/>{t('unified-config-open-first')}</button>
  <button type="button" class={css.ghost} disabled={!enabled.value()} onClick={()=>requestUnifiedMailbox()}><Icon name="mail"/>{t('common-open')}</button>
  <button type="button" class={css.ghost} aria-expanded={editing()} onClick={()=>setEditing(value=>!value)}><Icon name="settings"/>{t('unified-config-edit')}</button></div>
  <Show when={editing()}><div class={css.field}>
   <label class={css.check}><input type="checkbox" checked={enabled.value()} disabled={!enabled.entry()||enabled.locked()||enabled.saving()} onChange={event=>void enabled.set(event.currentTarget.checked)}/>{t('unified-mail-virtual-account')}</label><LockNote locked={enabled.locked()||first.locked()}/>
   <label class={css.field}>{t('unified-config-name')}<input class={css.input} value={name.value()} maxLength={128} disabled={!name.entry()||name.locked()||name.saving()} onChange={event=>void name.set(event.currentTarget.value)}/></label>
   <p class={css.meta}>{t('unified-config-help')}</p>
   <h4>{t('unified-config-accounts')}</h4><div class={layout.choices}><For each={choices()??[]}>{item=><label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={canAccount(unifiedAccountKey(item.target))&&has(included.value(),unifiedAccountKey(item.target))} disabled={!included.entry()||included.locked()||included.saving()||!canAccount(unifiedAccountKey(item.target))} onChange={event=>toggle(included,unifiedAccountKey(item.target),event.currentTarget.checked,(choices()??[]).map(item=>unifiedAccountKey(item.target)))}/><span>{item.target.label}</span></label>}</For></div>
   <h4>{t('unified-config-folders')}</h4><For each={choices()??[]}>{item=><Show when={canAccount(unifiedAccountKey(item.target))&&has(included.value(),unifiedAccountKey(item.target))}><details><summary>{item.target.label}</summary><div class={layout.choices}><For each={item.folders}>{folder=>{const key=unifiedFolderKey(item.target,folder);const permitted=()=>has(allowedFolders.value(),key,folder.role);return <label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={permitted()&&has(folders.value(),key,folder.role)} disabled={!folders.entry()||folders.locked()||folders.saving()||!permitted()} onChange={event=>toggle(folders,key,event.currentTarget.checked,(choices()??[]).flatMap(item=>item.folders.map(folder=>unifiedFolderKey(item.target,folder))))}/><span>{folder.name}</span></label>;}}</For></div></details></Show>}</For>
   <Show when={choices.loading}><p role="status">{t('common-loading')}</p></Show><Show when={choices.error}><p role="alert">{t('linked-accounts-error')}</p></Show>
  </div></Show>
 </section></Show>;
}
