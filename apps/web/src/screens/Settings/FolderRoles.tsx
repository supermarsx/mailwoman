import {For,Show,createSignal,type Accessor,type JSX} from 'solid-js';
import {useSetting} from './useSetting.ts';
import {LockNote} from './LockNote.tsx';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {t} from '../../i18n';
import {folderPathLabel,type PreferenceFolder} from './folderPreferenceModel.ts';
import * as css from './styles.css.ts';
import * as own from './FolderRoles.css.ts';
export const SPECIAL_ROLES=['inbox','drafts','sent','trash','junk','archive'] as const;
type Target={id:string}|{path:string}|null;
function target(value:unknown):value is Target {
 if(value===null)return true;
 if(typeof value!=='object'||Array.isArray(value))return false;
 const object=value as Record<string,unknown>;const keys=Object.keys(object);
 return keys.length===1&&(keys[0]==='id'||keys[0]==='path')&&typeof object[keys[0]]==='string';
}
function RoleField(props:{role:typeof SPECIAL_ROLES[number];folders:Accessor<readonly PreferenceFolder[]>;onSaved?:()=>Promise<void>}):JSX.Element {
 const setting=useSetting<Target>(`mail.folderRole.${props.role}`,null,target);
 const [applying,setApplying]=createSignal(false);const [refreshError,setRefreshError]=createSignal(false);
 const disabled=()=>!setting.entry()||setting.locked()||setting.saving()||applying();
 const current=()=>setting.value();
 const value=()=>{const selected=current();return selected===null?'auto':'id'in selected?`id:${selected.id}`:`path:${selected.path}`;};
 const configuredPath=()=>{const selected=current();return selected&&'path'in selected?selected.path:null;};
 const unresolved=()=>{const selected=current();if(!selected||disabled())return false;const folder=props.folders().find(folder=>'id'in selected?folder.id===selected.id:folder.mailwomanUpstreamName===selected.path);return !folder||folder.role!==props.role;};
 async function save(value:Target|undefined):Promise<void>{
  const owner=preferenceAccount();if(!owner||!preferenceAccountIsCurrent(owner))return;
  setApplying(true);setRefreshError(false);
  try{const saved=value===undefined?await setting.reset():await setting.set(value);if(saved&&preferenceAccountIsCurrent(owner))await props.onSaved?.();}
  catch{if(preferenceAccountIsCurrent(owner))setRefreshError(true);}
  finally{setApplying(false);}
 }
 return <div class={css.field}>
  <label for={`folder-role-${props.role}`}>{t(`settings-folder-role-${props.role}`)}</label>
  <select id={`folder-role-${props.role}`} class={`${css.input} ${own.select}`} value={value()} disabled={disabled()} aria-describedby="folder-role-help" onChange={event=>{const selected=event.currentTarget.value;if(selected==='auto')void save(null);else if(selected.startsWith('id:'))void save({id:selected.slice(3)});}}>
   <option value="auto">{t('settings-folder-role-auto')}</option>
   <Show when={configuredPath()}>{path=><option value={`path:${path()}`}>{t('settings-folder-role-path',{path:path()})}</option>}</Show>
   <For each={props.folders().filter(folder=>folder.mailwomanSearchQuery===undefined)}>{folder=><option value={`id:${folder.id}`}>{folderPathLabel(folder)}</option>}</For>
   <Show when={current()&&'id'in current()!&&!props.folders().some(folder=>`id:${folder.id}`===value())}><option value={value()}>{t('settings-folder-role-missing')}</option></Show>
  </select>
  <LockNote locked={setting.locked()}/>
  <Show when={unresolved()}><p role="status" class={css.meta}>{t('settings-folder-role-conflict')}</p></Show>
  <Show when={setting.error()||refreshError()}><p role="alert">{setting.error()||t('settings-folder-role-refresh-error')}</p></Show>
  <div class={css.actions}><button type="button" class={css.ghost} disabled={disabled()} onClick={()=>void save(undefined)}>{t('settings-folder-role-inherit')}</button></div>
 </div>;
}
export function FolderRoles(props:{folders:Accessor<readonly PreferenceFolder[]>;onSaved?:()=>Promise<void>}):JSX.Element {
 const supported=()=>props.folders().some(folder=>folder.mailwomanUpstreamName!==undefined);
 return <details class={own.panel}><summary>{t('settings-folder-role-title')}</summary>
  <p id="folder-role-help" class={css.prose}>{t('settings-folder-role-help')}</p>
  <Show when={supported()} fallback={<p class={css.meta}>{t('settings-folder-role-unsupported')}</p>}>
   <div class={own.grid}><For each={SPECIAL_ROLES}>{role=><RoleField role={role} folders={props.folders} {...(props.onSaved?{onSaved:props.onSaved}:{})}/>}</For></div>
  </Show>
 </details>;
}
