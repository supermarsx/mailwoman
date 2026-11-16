import {resourceValue} from '../../components/resourceValue.ts';
import * as table from './linkedAccounts.css.ts';
import {createResource,createSignal,createEffect,on,For,Show,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {MailboxAvatarEditor} from './MailboxAvatarEditor.tsx';
import {workspaceMailboxes,workspaceFolders} from '../../api/workspaceMail.ts';
import {unifiedAccountKey,unifiedFolderKey,parseUnifiedAvatar,parseUnifiedGroups,type UnifiedAccountGroup} from '../../state/unifiedAccountPreferences.ts';
import {requestUnifiedMailbox} from '../../state/unifiedMailbox.ts';
import {loadCatalog,t} from '../../i18n';
import {Icon} from '../../components/Icon.tsx';
import {LockNote} from './LockNote.tsx';
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
import * as css from './styles.css.ts';
import * as layout from './linkedSettingsInheritance.css.ts';
void loadCatalog('workspace-mail');
export function UnifiedAccountManager(props:{list:LinkedMailboxList;onChanged:()=>void;page?:boolean}):JSX.Element{
 const enabled=useSetting<boolean>('mail.unifiedAccount',false),first=useSetting<boolean>('mail.unifiedAccount.openFirst',false),name=useSetting<string>('mail.unifiedAccount.name',''),avatar=useSetting<string>('mail.unifiedAccount.avatar','');
 const groups=useSetting<string>('mail.unifiedAccount.groups','[]'),primaryGroup=useSetting<string>('mail.unifiedAccount.primaryGroup','');
 const [editingGroup,setEditingGroup]=createSignal<string>();
 const groupRows=()=>parseUnifiedGroups(groups.value());
 const updateGroup=(id:string,patch:Partial<UnifiedAccountGroup>)=>groups.set(JSON.stringify(groupRows().map(group=>group.id===id?{...group,...patch}:group)));
 const addGroup=()=>{const id=crypto.randomUUID();void groups.set(JSON.stringify([...groupRows(),{id,name:`${t('unified-mail-title')} ${groupRows().length+1}`,accounts:[],folders:['*']}])).then(saved=>{if(saved)setEditingGroup(id);});};
 const included=useSetting<string[]>('mail.unifiedAccount.accounts',['*']),folders=useSetting<string[]>('mail.unifiedAccount.folders',['*']);
 const allowed=useSetting<string[]>('mail.unifiedAccount.allowedAccounts',['*']),allowedFolders=useSetting<string[]>('mail.unifiedAccount.allowedFolders',['*']);
 createEffect(on(()=>JSON.stringify([enabled.value(),first.value(),name.value(),avatar.value(),included.value(),folders.value(),groups.value(),primaryGroup.value()]),()=>props.onChanged(),{defer:true}));
 const [editing,setEditing]=createSignal(props.page===true);
 const [choices]=createResource(()=>editing()||!!editingGroup(),async open=>{if(!open)return [];const targets=await workspaceMailboxes(null);const controller=new AbortController();return Promise.all(targets.map(async target=>({target,folders:await workspaceFolders(target,controller.signal)})));});
 const choiceRows=()=>resourceValue(choices)??[];
 const label=()=>name.value().trim()||t('unified-mail-title');
 const has=(items:string[],key:string,role?:string|null)=>items.includes('*')||items.includes(key)||!!role&&items.includes(`role:${role}`);
 const canAccount=(key:string)=>has(allowed.value(),key);
 const toggle=(setting:typeof included,key:string,checked:boolean,all:string[])=>{const current=setting.value().includes('*')?all:setting.value();void setting.set(checked?[...new Set([...current,key])]:current.filter(value=>value!==key));};
 return <Show when={props.list.policy.enabled&&props.list.accounts.some(account=>account.status==='ready')}><section class={props.page?css.stack:css.section} data-testid="unified-account-manager">
  <div class={`${css.actions} ${table.unifiedHeader}`}><MailboxAvatarEditor mailboxKey="@unified" label={label()} address="@unified" avatar={parseUnifiedAvatar(avatar.value())} disabled={!avatar.entry()||avatar.locked()||avatar.saving()} onSave={value=>avatar.set(value?JSON.stringify(value):'')}/><div><h3 class={css.heading}>{label()}</h3><p class={css.meta}>{t('unified-mail-virtual-account')}</p></div>
  <button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-pressed={first.value()&&primaryGroup.value()===''} disabled={!first.entry()||!primaryGroup.entry()||first.locked()||primaryGroup.locked()||primaryGroup.saving()||first.saving()||!enabled.value()} onClick={()=>{const next=!(first.value()&&primaryGroup.value()==='');void primaryGroup.set('').then(saved=>saved?first.set(next):false);}}><Icon name={first.value()&&primaryGroup.value()===''?'star':'starOutline'}/>{t('unified-config-open-first')}</button>
  <button type="button" class={`${css.ghost} ${table.unifiedAction}`} disabled={!enabled.value()} onClick={()=>requestUnifiedMailbox(undefined,null)}><Icon name="mail"/>{t('common-open')}</button>
  <Show when={!props.page}><button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-expanded={editing()} onClick={()=>setEditing(value=>!value)}><Icon name="settings"/>{t('unified-config-edit')}</button></Show></div>
  <Show when={editing()}><div class={css.field}>
   <label class={css.check}><input type="checkbox" checked={enabled.value()} disabled={!enabled.entry()||enabled.locked()||enabled.saving()} onChange={event=>void enabled.set(event.currentTarget.checked)}/>{t('unified-mail-virtual-account')}</label><LockNote locked={enabled.locked()||first.locked()}/>
   <label class={css.field}>{t('unified-config-name')}<input class={css.input} value={name.value()} maxLength={128} disabled={!name.entry()||name.locked()||name.saving()} onChange={event=>void name.set(event.currentTarget.value)}/></label>
   <p class={css.meta}>{t('unified-config-help')}</p>
   <h4>{t('unified-config-accounts')}</h4><div class={layout.choices}><For each={choiceRows()}>{item=><label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={canAccount(unifiedAccountKey(item.target))&&has(included.value(),unifiedAccountKey(item.target))} disabled={!included.entry()||included.locked()||included.saving()||!canAccount(unifiedAccountKey(item.target))} onChange={event=>toggle(included,unifiedAccountKey(item.target),event.currentTarget.checked,(choiceRows()).map(item=>unifiedAccountKey(item.target)))}/><span>{item.target.label}</span></label>}</For></div>
   <h4>{t('unified-config-folders')}</h4><For each={choiceRows()}>{item=><Show when={canAccount(unifiedAccountKey(item.target))&&has(included.value(),unifiedAccountKey(item.target))}><details><summary>{item.target.label}</summary><div class={layout.choices}><For each={item.folders}>{folder=>{const key=unifiedFolderKey(item.target,folder);const permitted=()=>has(allowedFolders.value(),key,folder.role);return <label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={permitted()&&has(folders.value(),key,folder.role)} disabled={!folders.entry()||folders.locked()||folders.saving()||!permitted()} onChange={event=>toggle(folders,key,event.currentTarget.checked,(choiceRows()).flatMap(item=>item.folders.map(folder=>unifiedFolderKey(item.target,folder))))}/><span>{folder.name}</span></label>;}}</For></div></details></Show>}</For>
   <Show when={choices.loading}><p role="status">{t('common-loading')}</p></Show><Show when={choices.error}><p role="alert">{t('linked-accounts-error')}</p></Show>
  </div></Show>
  <div class={`${css.actions} ${table.unifiedHeader}`}><h3 class={css.heading}>{t('unified-config-groups')}</h3><button type="button" class={`${css.ghost} ${table.unifiedAction}`} disabled={!groups.entry()||groups.locked()||groups.saving()||groupRows().length>=16} onClick={addGroup}><Icon name="plus"/>{t('unified-config-add-group')}</button><LockNote locked={groups.locked()}/></div>
  <For each={groupRows()}>{group=><div class={css.section} data-unified-group={group.id}>
   <div class={`${css.actions} ${table.unifiedHeader}`}><MailboxAvatarEditor mailboxKey={`@unified:${group.id}`} label={group.name} address={`@unified:${group.id}`} avatar={parseUnifiedAvatar(group.avatar)} disabled={groups.locked()||groups.saving()} onSave={value=>updateGroup(group.id,{avatar:value?JSON.stringify(value):''})}/><strong>{group.name}</strong>
    <button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-label={t('unified-config-open-first')} aria-pressed={first.value()&&primaryGroup.value()===group.id} disabled={!enabled.value()||first.locked()||primaryGroup.locked()||first.saving()||primaryGroup.saving()} onClick={()=>void primaryGroup.set(group.id).then(saved=>saved?first.set(true):false)}><Icon name={first.value()&&primaryGroup.value()===group.id?'star':'starOutline'}/></button>
    <button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-label={t('common-open')} disabled={!enabled.value()} onClick={()=>requestUnifiedMailbox(undefined,group.id)}><Icon name="mail"/></button>
    <button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-label={t('unified-config-edit')} aria-expanded={editingGroup()===group.id} onClick={()=>setEditingGroup(id=>id===group.id?undefined:group.id)}><Icon name="settings"/></button>
    <button type="button" class={`${css.ghost} ${table.unifiedAction}`} aria-label={t('common-delete')} disabled={groups.locked()||groups.saving()} onClick={()=>void groups.set(JSON.stringify(groupRows().filter(row=>row.id!==group.id)))}><Icon name="delete"/></button>
   </div>
   <Show when={editingGroup()===group.id}><div class={css.field}>
    <label class={css.field}>{t('unified-config-name')}<input class={css.input} value={group.name} maxLength={128} disabled={groups.locked()||groups.saving()} onChange={event=>void updateGroup(group.id,{name:event.currentTarget.value.trim()||group.name})}/></label>
    <p class={css.meta}>{t('unified-config-help')}</p><h4>{t('unified-config-accounts')}</h4>
    <div class={layout.choices}><For each={choiceRows()}>{item=>{const key=unifiedAccountKey(item.target);return <label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={canAccount(key)&&has(group.accounts,key)} disabled={groups.locked()||groups.saving()||!canAccount(key)} onChange={event=>{const current=group.accounts.includes('*')?(choiceRows()).map(item=>unifiedAccountKey(item.target)):group.accounts;void updateGroup(group.id,{accounts:event.currentTarget.checked?[...new Set([...current,key])]:current.filter(id=>id!==key)});}}/><span>{item.target.label}</span></label>;}}</For></div>
    <h4>{t('unified-config-folders')}</h4><For each={choiceRows()}>{item=><Show when={canAccount(unifiedAccountKey(item.target))&&has(group.accounts,unifiedAccountKey(item.target))}><details><summary>{item.target.label}</summary><div class={layout.choices}><For each={item.folders.filter(folder=>folder.mailwomanSearchQuery===undefined)}>{folder=>{const key=unifiedFolderKey(item.target,folder),permitted=()=>has(allowedFolders.value(),key,folder.role);return <label class={`${css.check} ${layout.choice}`}><input type="checkbox" checked={permitted()&&has(group.folders,key,folder.role)} disabled={groups.locked()||groups.saving()||!permitted()} onChange={event=>{const current=group.folders.includes('*')?(choiceRows()).flatMap(item=>item.folders.map(folder=>unifiedFolderKey(item.target,folder))):group.folders;void updateGroup(group.id,{folders:event.currentTarget.checked?[...new Set([...current,key])]:current.filter(id=>id!==key)});}}/><span>{folder.name}</span></label>;}}</For></div></details></Show>}</For>
    <Show when={choices.loading}><p role="status">{t('common-loading')}</p></Show><Show when={choices.error}><p role="alert">{t('linked-accounts-error')}</p></Show>
   </div></Show>
  </div>}</For><Show when={groups.error()}><p role="alert">{groups.error()}</p></Show>

 </section></Show>;
}
