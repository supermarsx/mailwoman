import {createEffect,createSignal,For,Show,onMount,onCleanup,type JSX} from 'solid-js';
import {t,loadCatalog} from '../../i18n';
import {registerNavigationGuard} from '../../shell/navigationGuard.ts';
import {useEditorGuard} from '../../screens/Settings/entityEditor.tsx';
import * as css from './contacts.css.ts';

export function GroupEditor(props:{books:{id:string;name:string}[];initialBook:string;initialName?:string;editing?:boolean;onSave:(name:string,book:string)=>Promise<string|null>;onSaved:()=>void;onCancel:()=>void}):JSX.Element{
 onMount(()=>{void loadCatalog('settings');});
 const [name,setName]=createSignal(props.initialName??''),[book,setBook]=createSignal(props.initialBook),[busy,setBusy]=createSignal(false),[error,setError]=createSignal(false),[saved,setSaved]=createSignal(false);
 createEffect(()=>{if(!book())setBook(props.initialBook||props.books[0]?.id||'');});
 let alive=true;onCleanup(()=>{alive=false;});
 const guard=useEditorGuard(()=>busy()||(!saved()&&(name()!==(props.initialName??'')||book()!==props.initialBook)),busy);onCleanup(registerNavigationGuard(guard.confirm));
 async function submit(event:Event):Promise<void>{event.preventDefault();if(busy()||!name().trim()||!book())return;setBusy(true);setError(false);try{const id=await props.onSave(name().trim(),book());if(!alive)return;if(id){setSaved(true);props.onSaved();}else setError(true);}catch{if(alive)setError(true);}finally{if(alive)setBusy(false);}}
 return <><form class={css.card} aria-label={t(props.editing?'contacts-edit-group':'contacts-new-group')} aria-busy={busy()} onSubmit={event=>void submit(event)}><h1 class={css.cardName}>{t(props.editing?'contacts-edit-group':'contacts-new-group')}</h1><p>{t(props.editing?'contacts-group-edit-help':'contacts-group-editor-help')}</p>
 <fieldset class={css.editorSection} disabled={busy()}><legend>{t('contacts-group-details')}</legend><div class={css.fieldGroup}><label for="group-name">{t(props.editing?'contacts-group-name':'contacts-new-group-name')}</label><input id="group-name" class={css.input} required maxlength={255} value={name()} onInput={event=>setName(event.currentTarget.value)}/></div><div class={css.fieldGroup}><label for="group-book">{t('contacts-address-books')}</label><select id="group-book" class={css.input} disabled={props.editing} required value={book()} onChange={event=>setBook(event.currentTarget.value)}><For each={props.books}>{item=><option value={item.id}>{item.name}</option>}</For></select></div></fieldset>
 <Show when={error()}><p role="alert">{t('contacts-group-save-error')}</p></Show><div class={css.actions}><button type="button" class={css.buttonGhost} disabled={busy()} onClick={()=>void guard.leave(props.onCancel)}>{t('common-cancel')}</button><button type="submit" class={css.button} disabled={busy()||!name().trim()||!book()}>{t(props.editing?'common-save':'contacts-create')}</button></div></form>{guard.dialog()}</>;
}
