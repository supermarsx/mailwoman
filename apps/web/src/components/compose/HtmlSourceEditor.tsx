import {EditorCounts} from './EditorCounts.tsx';
import {createEffect,createSignal,createUniqueId,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {normalizeHtmlSource} from './htmlSource.ts';
import {NativeHtmlVisualEditor,type NativeHtmlVisualApi} from './NativeHtmlVisualEditor.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';
import {t} from '../../i18n/index.ts';
import * as css from './nativeHtmlEditor.css.ts';
/** Source editing preserves native email layout through the HTML allowlist. Never renders raw HTML; `onChange` also gives the source as typed.
 * `externalText` (the composer's plain body) is reconciled in; `embedded` keeps raster `data:` images (signatures). */
export default function HtmlSourceEditor(props:{initialHtml:string;externalText?:()=>string;owned:()=>ReadonlySet<string>;embedded?:boolean;preview?:(cid:string)=>Promise<string|null>;upload?:(files:File[])=>Promise<{cid:string;alt:string}[]>;onChange:(html:string,text:string,source:string)=>void;onReady:(api:RichTextApi)=>void}):JSX.Element {
 const [source,setSource]=createSignal(props.initialHtml);const [text,setText]=createSignal('');const [processing,setProcessing]=createSignal(false);const [failed,setFailed]=createSignal(false);let field!:HTMLTextAreaElement;let lastText=props.externalText?.()??'';
 let revision=0,disposed=false,pending:Promise<void>=Promise.resolve();
 const [view,setView]=createSignal<'visual'|'source'>('visual');const [normalized,setNormalized]=createSignal('');const [visualRevision,setVisualRevision]=createSignal(0);let visual:NativeHtmlVisualApi|undefined;
 const viewId=createUniqueId();
 onCleanup(()=>{disposed=true;revision++;});
 function publish(value:string,updateVisual=true):void{
  const current=++revision;setProcessing(true);setFailed(false);
  pending=normalizeHtmlSource(value,props.owned(),props.embedded===true).then(result=>{
   if(disposed||current!==revision)return;
   lastText=result.text;setText(lastText);setNormalized(result.html);if(updateVisual)setVisualRevision(current=>current+1);props.onChange(result.html,lastText,value);
  }).catch(error=>{if(!disposed&&current===revision)setFailed(true);throw error;}).finally(()=>{if(!disposed&&current===revision)setProcessing(false);});
  // A textarea edit has no awaiting caller; save/send await the same promise.
  void pending.catch(()=>undefined);
 }
 async function settle():Promise<void>{let latest:Promise<void>;do{latest=pending;try{await latest;}catch(error){if(latest===pending)throw error;}}while(latest!==pending);}
 async function whenReady():Promise<void>{await settle();if(view()==='visual'&&visual){const value=await visual.capture();const safe=await normalizeHtmlSource(value,props.owned(),props.embedded===true);if(safe.html!==normalized()){setSource(value);publish(value,false);await settle();}}}
 function replace(value:string):void{setSource(value);publish(value);}
 async function changeView(next:'visual'|'source'):Promise<void>{if(next===view())return;try{await whenReady();visual=undefined;setView(next);}catch{setFailed(true);if(next==='source'){visual=undefined;setView(next);}}}
 function tabKey(event:KeyboardEvent):void{if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;event.preventDefault();const group=event.currentTarget as HTMLElement,next=event.key==='Home'?'visual':event.key==='End'?'source':view()==='visual'?'source':'visual';void changeView(next).then(()=>{if(view()===next)group.querySelector<HTMLElement>(`[data-view="${next}"]`)?.focus();});}
 onMount(()=>{publish(source());props.onReady({setHtml:replace,appendHtml:value=>replace(source()+value),whenReady,focus:()=>view()==='visual'?visual?.focus():field.focus(),refreshImages:()=>visual?.refreshImages()});});
 createEffect(()=>{const external=props.externalText;if(external===undefined)return;const value=external();if(value!==lastText){lastText=value;replace(`<p>${value.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/\n/g,'<br>')}</p>`);}});
 return <div class="compose-source"><div class={css.views} role="tablist" aria-label={t('mail-compose-format-html')} onKeyDown={tabKey}><button id={`${viewId}-visual`} type="button" class={`btn btn--ghost ${css.viewTab}`} role="tab" data-view="visual" aria-controls={`${viewId}-panel`} aria-selected={view()==='visual'} tabIndex={view()==='visual'?0:-1} onClick={()=>void changeView('visual')}>{t('mail-compose-html-visual')}</button><button id={`${viewId}-source`} type="button" class={`btn btn--ghost ${css.viewTab}`} role="tab" data-view="source" aria-controls={`${viewId}-panel`} aria-selected={view()==='source'} tabIndex={view()==='source'?0:-1} onClick={()=>void changeView('source')}>{t('mail-compose-html-source')}</button></div><div role="tabpanel" id={`${viewId}-panel`} aria-labelledby={`${viewId}-${view()}`}><Show when={view()==='source'} fallback={<NativeHtmlVisualEditor html={normalized()} revision={visualRevision()} owned={props.owned} {...(props.embedded===undefined?{}:{embedded:props.embedded})} {...(props.preview?{preview:props.preview}:{})} {...(props.upload?{upload:props.upload}:{})} onReady={api=>{visual=api;}} onInput={value=>{setSource(value);publish(value,false);}}/>}><p>{t('mail-compose-html-help')}</p><textarea ref={field} aria-label={t('mail-compose-html-source')} aria-busy={processing()} spellcheck={false} maxlength={786432} value={source()} onInput={event=>replace(event.currentTarget.value)}/></Show></div><span role="status">{failed()?t('mail-compose-html-failed'):processing()?t('mail-compose-html-processing'):''}</span><Show when={failed()}><button type="button" class="btn" onClick={()=>publish(source())}>{t('common-retry')}</button></Show><EditorCounts text={props.externalText?.()??text()}/></div>;
}
