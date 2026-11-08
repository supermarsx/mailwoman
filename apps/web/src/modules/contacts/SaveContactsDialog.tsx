// The chooser behind "Save contacts from this message" when there is more than
// one candidate: each participant with a checkbox, an editable name, a role
// badge, an address book and an optional group. Opened imperatively (it mounts
// into document.body), so any menu can open it without a host component.
import {createMemo,createSignal,For,onCleanup,onMount,Show,type JSX,type Signal} from 'solid-js';
import {render} from 'solid-js/web';
import type {Email} from '../../api/jmap-types.ts';
import {createFocusTrap} from '../../components/a11y';
import {Spinner} from '../../components/spinner/Spinner.tsx';
import {loadCatalog,t} from '../../i18n';
import {bookWritable,collectParticipants,defaultChecked,displayNameFor,pickBook,saveSettings,type Participant,type Role} from './saveFromMessage.ts';
import {contactOwner,announceSaved,fetchMissingFields,openContactById,ownAddresses,resolveParticipants,saveItems,type DialogRequest,type Resolved} from './saveFromMessage.flows.ts';
import * as css from './saveContacts.css.ts';

interface Line {participant:Participant;checked:Signal<boolean>;name:Signal<string>;book:Signal<string>;group:Signal<string>}

const roleLabel=(role:Role):string=>t(`contacts-save-role-${role}`);

export function SaveContactsDialog(props:{request:DialogRequest;onClose:()=>void}):JSX.Element {
 const {app,deps}=props.request;
 const current=contactOwner(app);
 const [element,setElement]=createSignal<HTMLElement>();
 const [resolved,setResolved]=createSignal<Resolved>();
 const [lines,setLines]=createSignal<Line[]>([]);
 const [showAutomated,setShowAutomated]=createSignal(false);
 const [busy,setBusy]=createSignal(false);
 let alive=true;
 onCleanup(()=>{alive=false;});
 const titleId='save-contacts-title';
 createFocusTrap(element,{onEscape:props.onClose});
 onMount(()=>{
  void loadCatalog('contacts');
  void (async()=>{
   let loaded:readonly Email[]=props.request.messages;
   try{loaded=await (deps.fetchFields??fetchMissingFields)(app,props.request.messages);}catch{/* the fields on the list rows still work */}
   if(!alive||!current())return;
   const all=collectParticipants(loaded,ownAddresses(app)).filter(item=>!item.own);
   const next=await resolveParticipants(app,all).catch(()=>({fresh:all,saved:[],books:app.addressBooks()} as Resolved));
   if(!alive||!current())return;
   const book=pickBook(next.books,saveSettings.defaultBook())?.id??'';
   const scope=saveSettings.defaultScope();
   setLines(next.fresh.map(participant=>({participant,
    checked:createSignal(defaultChecked(participant,scope)&&!(saveSettings.skipAutomated()&&participant.automated)),
    name:createSignal(displayNameFor(participant.name,participant.email)),book:createSignal(book),group:createSignal('')})));
   setResolved(next);
  })();
 });
 const visible=createMemo(()=>lines().filter(line=>showAutomated()||!(saveSettings.skipAutomated()&&line.participant.automated)));
 const hasAutomated=createMemo(()=>saveSettings.skipAutomated()&&lines().some(line=>line.participant.automated));
 const chosen=createMemo(()=>visible().filter(line=>line.checked[0]()&&line.name[0]().trim()!==''&&line.book[0]()!==''));
 const books=()=>resolved()?.books??[];
 const groupsFor=(book:string)=>app.contactGroups().filter(group=>group.addressBookId===book);
 async function submit():Promise<void> {
  if(!current()){props.onClose();return;}
  if(busy()||!chosen().length)return;
  setBusy(true);
  const picked=chosen();
  try{
  const done=await saveItems(app,picked.map(line=>({email:line.participant.email,name:line.name[0]().trim(),bookId:line.book[0](),...(line.group[0]()?{groupId:line.group[0]()}:{})})));
  if(!alive||!current())return;
  const first=picked[0]?.book[0]()??'';
  announceSaved(app,done,picked.length,books().find(book=>book.id===first)?.name??'');
  props.onClose();
  }catch{if(alive&&current())app.showToast('error',t('contacts-save-failed'));}
  finally{if(alive)setBusy(false);}
 }
 return <div class={css.backdrop} onMouseDown={event=>{if(event.target===event.currentTarget)props.onClose();}}>
  <div class={css.dialog} role="dialog" aria-modal="true" aria-labelledby={titleId} aria-busy={!resolved()} ref={setElement} tabindex="-1">
   <h2 class={css.title} id={titleId}>{t('contacts-save-dialog-title')}</h2>
   <Show when={resolved()} fallback={<Spinner label={t('contacts-save-loading')}/>}>
    <Show when={hasAutomated()}>
     <label><input type="checkbox" checked={showAutomated()} onChange={event=>setShowAutomated(event.currentTarget.checked)}/> {t('contacts-save-show-automated')}</label>
    </Show>
    <Show when={visible().length===0}><p class={css.muted}>{t('contacts-save-nothing')}</p></Show>
    <ul class={css.list}>
     <For each={visible()}>{line=>{
      const p=line.participant,id=`save-${p.email}`;
      return <li class={css.row}>
       <input type="checkbox" aria-label={t('contacts-save-pick',{email:p.email})} checked={line.checked[0]()} onChange={event=>line.checked[1](event.currentTarget.checked)}/>
       <div><For each={p.roles}>{role=><span class={css.badge}>{roleLabel(role)}</span>}</For><span class={css.email}>{p.email}</span></div>
       <label class={css.field} for={`${id}-name`}><span>{t('contacts-save-name')}</span><input id={`${id}-name`} type="text" value={line.name[0]()} onInput={event=>line.name[1](event.currentTarget.value)}/></label>
       <div class={css.wide}>
        <label class={css.field} for={`${id}-book`}><span>{t('contacts-save-book')}</span>
         <select id={`${id}-book`} value={line.book[0]()} onChange={event=>{line.book[1](event.currentTarget.value);line.group[1]('');}}>
          <For each={books()}>{book=><option value={book.id} disabled={!bookWritable(book)} selected={book.id===line.book[0]()}>{bookWritable(book)?book.name:t('contacts-save-book-readonly',{name:book.name})}</option>}</For>
         </select></label>
        <Show when={groupsFor(line.book[0]()).length>0}>
         <label class={css.field} for={`${id}-group`}><span>{t('contacts-save-group')}</span>
          <select id={`${id}-group`} value={line.group[0]()} onChange={event=>line.group[1](event.currentTarget.value)}>
           <option value="">{t('contacts-save-group-none')}</option>
           <For each={groupsFor(line.book[0]())}>{group=><option value={group.id}>{group.name}</option>}</For>
          </select></label>
        </Show>
       </div>
      </li>;}}</For>
    </ul>
    <Show when={(resolved()?.saved.length??0)>0}>
     <section aria-label={t('contacts-save-saved-heading')}>
      <h3 class={css.muted}>{t('contacts-save-saved-heading')}</h3>
      <ul class={css.list}><For each={resolved()!.saved}>{entry=><li><span class={css.badge}>{t('contacts-save-already-badge')}</span>{entry.card.name.full||entry.participant.email}{' '}
       <button type="button" onClick={()=>{props.onClose();(deps.openContact??openContactById)(app,entry.card.id);}}>{t('contacts-save-open-link')}</button></li>}</For></ul>
     </section>
    </Show>
   </Show>
   <div class={css.actions}>
    <button type="button" onClick={props.onClose}>{t('contacts-save-cancel')}</button>
    <button type="button" disabled={busy()||chosen().length===0} aria-busy={busy()||undefined} onClick={()=>void submit()}>{t('contacts-save-submit',{count:chosen().length})}</button>
   </div>
  </div>
 </div>;
}

/** Mounts the dialog into the page; closing disposes it. */
export function openSaveContactsDialog(request:DialogRequest):void {
 if(typeof document==='undefined')return;
 const host=document.createElement('div');
 document.body.appendChild(host);
 const dispose=render(()=><SaveContactsDialog request={request} onClose={()=>{dispose();host.remove();}}/>,host);
}
