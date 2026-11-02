import {For,createEffect,createSignal,on,onCleanup,onMount,type JSX} from 'solid-js';
import {t} from '../../i18n/index.ts';
import {nativeHtmlBody,normalizeHtmlSource} from './htmlSource.ts';
import * as css from './nativeHtmlEditor.css.ts';

export interface NativeHtmlVisualApi {capture:()=>Promise<string>;focus:()=>void;refreshImages:()=>void}
const FRAME_DOCUMENT='<!doctype html><html data-mw-native-document="true"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'none\'; style-src \'none\'; img-src data:; font-src data:; connect-src \'none\'; base-uri \'none\'; form-action \'none\'"></head><body></body></html>';
const commands=[['bold','mail-compose-rt-bold'],['italic','mail-compose-rt-italic'],['underline','mail-compose-rt-underline'],['insertUnorderedList','mail-compose-rt-bullet'],['insertOrderedList','mail-compose-rt-ordered'],['justifyLeft','mail-editor-align-left'],['justifyCenter','mail-editor-align-center'],['justifyRight','mail-editor-align-right']] as const;

/** An editor-specific isolation contract: the parent may edit this same-origin
 * document, while sandbox and CSP prohibit every child script. Message viewers
 * keep their separate opaque-origin sandbox. Clipboard/drop HTML is sanitized
 * before insertion, so it cannot briefly load resources before being cleaned. */
export function NativeHtmlVisualEditor(props:{html:string;revision:number;owned:()=>ReadonlySet<string>;embedded?:boolean;preview?:(cid:string)=>Promise<string|null>;upload?:(files:File[])=>Promise<{cid:string;alt:string}[]>;onInput:(html:string)=>void;onReady:(api:NativeHtmlVisualApi)=>void}):JSX.Element {
 let frame!:HTMLIFrameElement,editor:HTMLDivElement|undefined,disposed=false,documentRevision=0,changed=false;
 let styles:string[]=[];let undoListeners=()=>{};let pending:Promise<void>=Promise.resolve();
 const originals=new WeakMap<Element,Map<string,string>>();
 const [ready,setReady]=createSignal(false);
 const [failed,setFailed]=createSignal(false);
 let finishReady!:()=>void;
 const admitted=new Promise<void>(resolve=>{finishReady=resolve;});
 function serialize():string {
  if(!editor)return props.html;
  const clone=editor.cloneNode(true) as HTMLElement;
  const nodes=Array.from(editor.querySelectorAll('*')),copies=Array.from(clone.querySelectorAll('*'));
  nodes.forEach((node,index)=>{const copy=copies[index];if(!copy)return;for(const [name,value] of originals.get(node)??[]){if(name.startsWith('style:'))(copy as HTMLElement).style.setProperty(name.slice(6),value);else copy.setAttribute(name,value);}});
  return styles.map(style=>`<style>${style}</style>`).join('')+clone.innerHTML;
 }
 function emit():void {if(!disposed){changed=true;props.onInput(serialize());}}
 function previewImages(revision:number):void {
  if(!props.preview||!editor)return;
  for(const image of editor.querySelectorAll<HTMLImageElement>('img')){
   const src=originals.get(image)?.get('src')??'';if(!/^cid:/i.test(src))continue;
   const cid=src.slice(4);if(!props.owned().has(cid))continue;
   void props.preview(cid).then(preview=>{if(disposed||revision!==documentRevision||!image.isConnected||!preview||!/^data:image\/(png|jpeg|gif|webp);base64,[a-z0-9+/]+=*$/i.test(preview))return;image.src=preview;},()=>undefined);
  }
 }
 function installStyles(doc:Document):void {
  const view=doc.defaultView as (Window&{CSSStyleSheet:typeof CSSStyleSheet})|null;
  if(view){const sheet=new view.CSSStyleSheet();sheet.replaceSync('html,body{margin:0;background:white;color:#1c1e21}body{padding:12px;font:14px/1.5 system-ui,sans-serif}#mw-native-editor{min-height:12rem;outline:0}img{max-width:100%;height:auto}'+styles.join('\n').replace(/url\([^)]*\bcid:[^)]*\)/gi,'none'));doc.adoptedStyleSheets=[sheet];}
 }
 function loadDocument():void {
  const doc=frame.contentDocument;if(!doc||!editor)return;
  const revision=++documentRevision;
  changed=false;
  const body=nativeHtmlBody(props.html);
  styles=Array.from(body.querySelectorAll('style')).map(style=>style.textContent??'');
  body.querySelectorAll('style').forEach(style=>style.remove());
  const inlineStyles=Array.from(body.querySelectorAll<HTMLElement>('[style]')).map(element=>[element,element.getAttribute('style')??''] as const);
  const remembered=new Map<Element,Map<string,string>>();
  for(const element of body.querySelectorAll<HTMLElement>('*')){
   const original=new Map<string,string>();
   for(const name of ['src','background']){const value=element.getAttribute(name);if(value&&/^cid:/i.test(value)){original.set(name,value);element.removeAttribute(name);}}
   for(const name of Array.from(element.style)){const value=element.style.getPropertyValue(name);if(/url\([^)]*\bcid:/i.test(value))original.set(`style:${name}`,value);}
   if(original.size)remembered.set(element,original);
  }
  inlineStyles.forEach(([element])=>element.removeAttribute('style'));
  editor.replaceChildren(...Array.from(body.childNodes).map(node=>doc.importNode(node,true)));
  const imported=Array.from(editor.querySelectorAll<HTMLElement>('*'));
  const source=Array.from(body.querySelectorAll<HTMLElement>('*'));
  const stored=new Map(inlineStyles);
  source.forEach((element,index)=>{const target=imported[index];if(!target)return;const style=stored.get(element);if(style!==undefined)target.style.cssText=style.replace(/url\([^)]*\bcid:[^)]*\)/gi,'none');const original=remembered.get(element);if(original)originals.set(target,original);});
  // CSSOM installation works under the inherited application CSP without
  // permitting inline style/script markup in the child document.
  installStyles(doc);
  previewImages(revision);
 }
 function insert(event:ClipboardEvent|DragEvent):void {
  event.preventDefault();const data='clipboardData'in event?event.clipboardData:event.dataTransfer;
  if(!data||!editor)return;
  const clipboardHtml=data.getData('text/html'),plain=data.getData('text/plain'),files=Array.from(data.files??[]);
  if(!clipboardHtml&&!plain&&!files.length)return;
  const doc=editor.ownerDocument,selection=doc.getSelection(),range=selection?.rangeCount?selection.getRangeAt(0).cloneRange():null,revision=documentRevision;
  pending=pending.catch(()=>undefined).then(async()=>{
   setFailed(false);
   const escape=(value:string)=>value.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
   let raw=clipboardHtml||`<p>${escape(plain).replace(/\n/g,'<br>')}</p>`;
   if(!clipboardHtml&&!plain&&files.length){
    if(files.some(file=>!['image/png','image/jpeg','image/gif','image/webp'].includes(file.type)))throw new Error('Unsupported image');
    if(props.upload){const images=await props.upload(files);if(!images.length)throw new Error('Image upload failed');raw=images.map(image=>`<img src="cid:${escape(image.cid)}" alt="${escape(image.alt)}">`).join('');}
    else if(props.embedded){const images=await Promise.all(files.map(file=>new Promise<string>((resolve,reject)=>{if(file.size>1024*1024){reject(new Error('Image too large'));return;}const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=()=>reject(reader.error);reader.readAsDataURL(file);})));raw=images.map((src,index)=>`<img src="${escape(src)}" alt="${escape(files[index]!.name)}">`).join('');}
    else throw new Error('Image upload unavailable');
   }
   const safe=await normalizeHtmlSource(raw,props.owned(),props.embedded===true);
   if(disposed||revision!==documentRevision||!editor)return;
   editor.focus();
   // Import allowlisted nodes without a child-realm HTML string sink. That
   // document intentionally has no Trusted Types policy or executable script.
   const parsed=nativeHtmlBody(safe.html);styles.push(...Array.from(parsed.querySelectorAll('style')).map(style=>style.textContent??''));parsed.querySelectorAll('style').forEach(style=>style.remove());
   const remembered=new Map<Element,Map<string,string>>();
   for(const element of parsed.querySelectorAll<HTMLElement>('*')){
    const original=new Map<string,string>();
    for(const name of ['src','background']){const value=element.getAttribute(name);if(value&&/^cid:/i.test(value)){original.set(name,value);element.removeAttribute(name);}}
    for(const name of Array.from(element.style)){const value=element.style.getPropertyValue(name);if(/url\([^)]*\bcid:/i.test(value)){original.set(`style:${name}`,value);element.style.setProperty(name,value.replace(/url\([^)]*\bcid:[^)]*\)/gi,'none'));}}
    if(original.size)remembered.set(element,original);
   }
   const source=Array.from(parsed.querySelectorAll<HTMLElement>('*')),inlineStyles=source.map(element=>element.getAttribute('style'));
   source.forEach(element=>element.removeAttribute('style'));
   const fragment=doc.createDocumentFragment();fragment.append(...Array.from(parsed.childNodes).map(node=>doc.importNode(node,true)));
   const targets=Array.from(fragment.querySelectorAll<HTMLElement>('*'));
   targets.forEach((element,index)=>{const style=inlineStyles[index];if(style!==null&&style!==undefined)element.style.cssText=style;const original=source[index]?remembered.get(source[index]!):undefined;if(original)originals.set(element,original);});
   const last=fragment.lastChild;if(!last)return;
   const insertion=range&&editor.contains(range.commonAncestorContainer)?range:doc.createRange();
   if(insertion!==range){insertion.selectNodeContents(editor);insertion.collapse(false);}
   insertion.deleteContents();insertion.insertNode(fragment);insertion.setStartAfter(last);insertion.collapse(true);selection?.removeAllRanges();selection?.addRange(insertion);
   installStyles(doc);previewImages(revision);emit();
  });
  void pending.catch(()=>{if(!disposed)setFailed(true);});
 }
 function loaded():void {
  const doc=frame.contentDocument;if(!doc||disposed||doc.documentElement.getAttribute('data-mw-native-document')!=='true')return;
  undoListeners();editor=doc.createElement('div');editor.id='mw-native-editor';editor.contentEditable='true';editor.tabIndex=0;editor.setAttribute('role','textbox');editor.setAttribute('aria-multiline','true');editor.setAttribute('aria-label',t('mail-compose-body'));doc.body.replaceChildren(editor);
  const input=()=>emit();const paste=(event:Event)=>insert(event as ClipboardEvent);const drop=(event:Event)=>insert(event as DragEvent);
  const before=(event:Event)=>{if(['insertFromPaste','insertFromDrop'].includes((event as InputEvent).inputType))event.preventDefault();};
  const click=(event:Event)=>{const target=event.target as Element|null;if(typeof target?.closest==='function'&&target.closest('a'))event.preventDefault();};
  const focus=(event:Event)=>{if(editor&&event.target&&editor.contains(event.target as Node)&&doc.activeElement!==editor)editor.focus({preventScroll:true});};
  const copy=(event:ClipboardEvent)=>{const selection=doc.getSelection();if(!editor||!event.clipboardData||!selection?.rangeCount||selection.isCollapsed)return;const range=selection.getRangeAt(0);if(!editor.contains(range.commonAncestorContainer))return;const inert=doc.implementation.createHTMLDocument(''),container=inert.createElement('div');container.append(inert.importNode(range.cloneContents(),true));const sourceImages=Array.from(editor.querySelectorAll('img')).filter(image=>range.intersectsNode(image));Array.from(container.querySelectorAll('img')).forEach((image,index)=>{const source=sourceImages[index];const src=source?originals.get(source)?.get('src'):undefined;if(src)image.setAttribute('src',src);});event.clipboardData.setData('text/html',styles.map(style=>`<style>${style}</style>`).join('')+container.innerHTML);event.clipboardData.setData('text/plain',selection.toString());event.preventDefault();if(event.type==='cut'){doc.execCommand('delete');emit();}};
  doc.addEventListener('input',input);doc.addEventListener('paste',paste);doc.addEventListener('drop',drop);doc.addEventListener('beforeinput',before);doc.addEventListener('click',click);doc.addEventListener('pointerdown',focus);doc.addEventListener('copy',copy);doc.addEventListener('cut',copy);
  undoListeners=()=>{doc.removeEventListener('input',input);doc.removeEventListener('paste',paste);doc.removeEventListener('drop',drop);doc.removeEventListener('beforeinput',before);doc.removeEventListener('click',click);doc.removeEventListener('pointerdown',focus);doc.removeEventListener('copy',copy);doc.removeEventListener('cut',copy);};
  loadDocument();setReady(true);finishReady();
 }
 createEffect(on(()=>props.revision,()=>{if(ready())loadDocument();},{defer:true}));
 onMount(()=>props.onReady({async capture(){await admitted;let latest:Promise<void>;do{latest=pending;await latest;}while(latest!==pending);if(disposed)throw new DOMException('Editor closed','AbortError');return changed?serialize():props.html;},focus:()=>editor?.focus(),refreshImages:()=>previewImages(documentRevision)}));
 onCleanup(()=>{disposed=true;finishReady();undoListeners();});
 return <><div class={css.toolbar} role="toolbar" aria-label={t('mail-compose-html-visual')}><For each={commands}>{([command,label])=><button type="button" class="btn btn--ghost" disabled={!ready()} onMouseDown={event=>event.preventDefault()} onClick={()=>{editor?.focus();frame.contentDocument?.execCommand(command);emit();}}>{t(label)}</button>}</For></div><iframe ref={frame} class={css.frame} title={t('mail-compose-html-visual')} sandbox="allow-same-origin" referrerPolicy="no-referrer" srcdoc={FRAME_DOCUMENT} onLoad={loaded}/><span role="status">{failed()?t('mail-compose-html-failed'):''}</span></>;
}
