import {createMemo,createSignal,For,Show,onCleanup,type JSX} from 'solid-js';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {loadCatalog,t} from '../../i18n';
import {settings} from '../../api/settings.ts';
import {createConfirm} from '../confirm.tsx';
import {toast} from '../toast/index.ts';
import {RibbonPopover} from './ribbonParts.tsx';
import {validResponseTemplates,type ResponseTemplate} from './responseTemplateModel.ts';
import * as css from './responseTemplates.css.ts';

export function ResponseTemplates(props:{disabled?:boolean;onInsert:(text:string,replace:boolean)=>void}):JSX.Element{
 void loadCatalog('response-templates');
 const [open,setOpen]=createSignal(false),[query,setQuery]=createSignal(''),[selected,setSelected]=createSignal<string|null>(null);
 const confirm=createConfirm();
 onCleanup(observePreferenceAccount(()=>{setOpen(false);setSelected(null);setQuery('');}));
 const enabled=()=>settings()['mail.templates.enabled']?.value===true;
 const personalAllowed=()=>settings()['mail.templates.allowPersonal']?.value===true;
 const library=(key:string):ResponseTemplate[]=>{const value=settings()[key]?.value;return validResponseTemplates(value)?value:[];};
 const shared=createMemo(()=>library('mail.templates.shared')),personal=createMemo(()=>personalAllowed()?library('mail.templates.personal'):[]);
 const choices=createMemo(()=>[...shared().map(row=>({...row,key:`shared:${row.id}`,shared:true})),...personal().map(row=>({...row,key:`personal:${row.id}`,shared:false}))]);
 const choice=createMemo(()=>choices().find(row=>row.key===selected()));
 const matches=createMemo(()=>choices().filter(row=>`${row.title}\n${row.body}`.toLocaleLowerCase().includes(query().trim().toLocaleLowerCase())));
 async function insert(replace:boolean):Promise<void>{
  const row=choice(),owner=preferenceAccount();if(props.disabled||!enabled()||!row)return;
  if(replace&&!await confirm.ask({title:t('response-templates-replace-title'),body:t('response-templates-replace-help'),confirmLabel:t('response-templates-replace'),danger:true}))return;
  if(props.disabled||!enabled()||preferenceAccount()!==owner||choice()?.key!==row.key||choice()?.body!==row.body)return;
  props.onInsert(row.body,replace);setOpen(false);toast.success(t('response-templates-inserted'));
 }
 return <><RibbonPopover label={t('response-templates-title')} icon="book" keepSelection disabled={props.disabled||!enabled()} open={open()} onOpenChange={setOpen}>
  <div class={css.panel}>
   <label class={css.field}>{t('response-templates-search')}<input class={css.control} type="search" value={query()} onInput={event=>setQuery(event.currentTarget.value)}/></label>
   <div class={css.list} role="group" aria-label={t('response-templates-library')}><For each={matches()}>{row=><button type="button" class="btn btn--ghost" aria-pressed={selected()===row.key} onClick={()=>setSelected(row.key)}>{row.title} · {t(row.shared?'response-templates-shared':'response-templates-personal')}</button>}</For></div>
   <Show when={choices().length===0}><p>{t('response-templates-empty-picker')}</p></Show>
   <Show when={choice()}>{row=><><pre class={css.preview}>{row().body}</pre><div class={css.actions}><button type="button" class="btn" disabled={props.disabled||!enabled()} onClick={()=>void insert(false)}>{t('response-templates-insert')}</button><button type="button" class="btn btn--ghost" disabled={props.disabled||!enabled()} onClick={()=>void insert(true)}>{t('response-templates-replace')}</button></div></>}</Show>
  </div>
 </RibbonPopover><confirm.Dialog/></>;
}
