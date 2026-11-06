import {createEffect,createSignal,on,onCleanup,onMount,useContext,type JSX} from 'solid-js';
import {AppContext} from '../../state/context.ts';
import type {ComposeBackground} from '../../theme/appearance.ts';
import {cssColorHex,readableColor,surfaceTone,toggledBackground,UNREADABLE_BELOW,contrastRatio} from './messageBackground.ts';
import {t} from '../../i18n/index.ts';
import {editorBodyBytes} from './editorCounts.ts';
import {ensureManropePreview} from './nativeFontPreview.ts';
import {nativeHtmlBody,normalizeHtmlSource} from './htmlSource.ts';
import {HTML_MESSAGE_FONT} from './htmlMessage.ts';
import * as css from './nativeHtmlEditor.css.ts';
import {NativeHtmlRibbon} from './NativeHtmlRibbon.tsx';
import {stripEditorMarkers} from './htmlMessage.ts';
import {createNativeHtmlCommands} from './nativeHtmlCommands.ts';
import {createNativeFormat,type NativePaint} from './nativeFormatCommands.ts';
import {createPainter} from './FormatControls.tsx';
import {useComposeFeature} from './composeFeatures.ts';
import {isClearKey,isGrowKey,isShrinkKey,mergeFormatting,type PasteMode} from './formatModel.ts';
import {materializeLists,dematerializeLists} from './listMarkers.ts';
import LIST_CSS from './compose-lists.css?raw';
import type {EditorSlots} from './EditorRibbon.tsx';
import type {RichTextApi} from './RichTextEditor.tsx';

export interface NativeHtmlBookmark {start:number;end:number;answer?:boolean}
export interface NativeHtmlVisualApi {capture:()=>Promise<string>;focus:()=>void;refreshImages:()=>void;undo:()=>void;redo:()=>void;historyDepth:()=>{undo:number;redo:number};bookmark:()=>NativeHtmlBookmark|null}
const FRAME_DOCUMENT='<!doctype html><html data-mw-native-document="true"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'none\'; style-src \'none\'; img-src data:; font-src data:; connect-src \'none\'; base-uri \'none\'; form-action \'none\'"></head><body></body></html>';

/** An editor-specific isolation contract: the parent may edit this same-origin
 * document, while sandbox and CSP prohibit every child script. Message viewers
 * keep their separate opaque-origin sandbox. Clipboard/drop HTML is sanitized
 * before insertion, so it cannot briefly load resources before being cleaned. */
export function NativeHtmlVisualEditor(props:{html:string;revision:number;htmlLimit?:number;reply?:boolean;owned:()=>ReadonlySet<string>;embedded?:boolean;preview?:(cid:string)=>Promise<string|null>;upload?:(files:File[])=>Promise<{cid:string;alt:string}[]>;ribbonSlots?:()=>EditorSlots;externalHistory?:boolean;bookmark?:NativeHtmlBookmark;onInput:(html:string)=>void;onReady:(api:NativeHtmlVisualApi)=>void}):JSX.Element {
 let frame!:HTMLIFrameElement,editor:HTMLDivElement|undefined,disposed=false,documentRevision=0,changed=false,focusPending=false;
 let styles:string[]=[],viewTimer:ReturnType<typeof setTimeout>|undefined,viewSheet:CSSStyleSheet|undefined;let undoListeners=()=>{};let pending:Promise<void>=Promise.resolve();let editRevision=0;
 const originals=new WeakMap<Element,Map<string,string>>();
 const [ready,setReady]=createSignal(false);
 const [failed,setFailed]=createSignal(false);
 const [error,setError]=createSignal(''),[version,setVersion]=createSignal(0),[historyIndex,setHistoryIndex]=createSignal(0),[historyLength,setHistoryLength]=createSignal(1),[api,setApi]=createSignal<RichTextApi|null>(null);
 const app=useContext(AppContext);
 const [localBackground,setLocalBackground]=createSignal<ComposeBackground>('theme'),[markers,setMarkers]=createSignal(false);
 const background=():ComposeBackground=>app?app.appearancePrefs().composeBackground:localBackground();
 const appTone=():'dark'|'light'=>app?.appearance()==='dark'?'dark':'light';
 const dark=()=>surfaceTone(background(),appTone())==='dark';
 const toggleBackground=()=>{const next=toggledBackground(background(),appTone());if(app)app.setAppearancePrefs({composeBackground:next});else setLocalBackground(next);};
 let history:string[]=[];
 const commands=createNativeHtmlCommands({root:()=>editor,change:()=>emit(),selection:()=>setVersion(value=>value+1),replaceNode:(from,to)=>{const original=originals.get(from);if(original)originals.set(to,original);}});
 const format=createNativeFormat(commands);
 const stepOn=useComposeFeature('format.fontSizeStep'),indentOn=useComposeFeature('format.indent'),clearOn=useComposeFeature('format.clearFormatting'),supOn=useComposeFeature('format.superscript'),subOn=useComposeFeature('format.subscript');
 const painter=createPainter<NativePaint>({capture:()=>format.capture(),apply:value=>format.applyPaint(value)});
 let finishReady!:()=>void;
 const admitted=new Promise<void>(resolve=>{finishReady=resolve;});
 function answerBlock():HTMLParagraphElement|null {return props.reply?editor?.querySelector(':scope > p, :scope > .mw-email-body > p')??null:null;}
 function focusEditor():void {if(!editor||!ready()){focusPending=true;return;}focusPending=false;if(props.reply&&!commands.range()){const range=editor.ownerDocument.createRange();range.selectNodeContents(answerBlock()??editor);range.collapse(true);commands.setRange(range);}else commands.restore();editor.focus({preventScroll:true});}
 function bookmark():NativeHtmlBookmark|null {const range=commands.range();if(!range||!editor)return null;const before=range.cloneRange();before.selectNodeContents(editor);before.setEnd(range.startContainer,range.startOffset);const answer=answerBlock();return {start:before.toString().length,end:before.toString().length+range.toString().length,...(answer&&answer.contains(range.commonAncestorContainer)&&!(answer.textContent??'').replace(/\u200b/g,'').trim()?{answer:true}:{})};}
 function restoreBookmark(value:NativeHtmlBookmark):void {if(!editor)return;const answer=value.answer?answerBlock():null;if(answer){const range=editor.ownerDocument.createRange();range.selectNodeContents(answer);range.collapse(true);commands.setRange(range);return;}const walker=editor.ownerDocument.createTreeWalker(editor,4),nodes:Text[]=[];let node:Node|null;while((node=walker.nextNode()))nodes.push(node as Text);const locate=(position:number,nextBoundary:boolean):[Node,number]=>{for(const node of nodes){if(position<node.length||(position===node.length&&!nextBoundary))return [node,position];position-=node.length;}return [editor!,editor!.childNodes.length];};const range=editor.ownerDocument.createRange();range.setStart(...locate(value.start,true));range.setEnd(...locate(value.end,value.start===value.end));commands.setRange(range);}
 function serialize():string {
  if(!editor)return props.html;
  const clone=editor.cloneNode(true) as HTMLElement;
  const nodes=Array.from(editor.querySelectorAll('*')),copies=Array.from(clone.querySelectorAll('*'));
  nodes.forEach((node,index)=>{const copy=copies[index];if(!copy)return;for(const [name,value] of originals.get(node)??[]){if(name.startsWith('style:'))(copy as HTMLElement).style.setProperty(name.slice(6),value);else copy.setAttribute(name,value);}});
  for(const answer of clone.querySelectorAll('[data-mw-native-answer]'))answer.removeAttribute('data-mw-native-answer');
  for(const shown of clone.querySelectorAll('[data-mw-vc]'))shown.removeAttribute('data-mw-vc');
  for(const caret of clone.querySelectorAll('[data-mw-native-caret]')){caret.removeAttribute('data-mw-native-caret');const text=caret.firstChild;if(text?.nodeType===3&&text.textContent?.startsWith('\u200b'))text.textContent=text.textContent.slice(1);if(!caret.textContent&&!caret.children.length)caret.remove();}
  materializeLists(clone);for(const item of Array.from(clone.querySelectorAll('[data-mw-checked]')))item.removeAttribute('data-mw-checked');
  return styles.map(style=>`<style>${style}</style>`).join('')+clone.innerHTML;
 }
 function record(value:string):void {if(history[historyIndex()]===value)return;history=history.slice(0,historyIndex()+1);history.push(value);if(history.length>101)history.shift();setHistoryIndex(history.length-1);setHistoryLength(history.length);}
 function emit():void {if(!disposed){editRevision++;changed=true;scheduleViewColors();const value=serialize();record(value);props.onInput(value);setVersion(value=>value+1);}}
 function moveHistory(direction:number):void {const next=historyIndex()+direction;if(next<0||next>=history.length)return;const selected=bookmark();setHistoryIndex(next);loadDocument(history[next]);if(selected)restoreBookmark(selected);changed=true;props.onInput(serialize());setVersion(value=>value+1);}
 function previewImages(revision:number):void {
  if(!props.preview||!editor)return;
  for(const image of editor.querySelectorAll<HTMLImageElement>('img')){
   const src=originals.get(image)?.get('src')??'';if(!/^cid:/i.test(src))continue;
   const cid=src.slice(4);if(!props.owned().has(cid))continue;
   void props.preview(cid).then(preview=>{if(disposed||revision!==documentRevision||!image.isConnected||!preview||!/^data:image\/(png|jpeg|gif|webp);base64,[a-z0-9+/]+=*$/i.test(preview))return;image.src=preview;},()=>undefined);
  }
 }

 // View-only automatic text colour (like Outlook): text whose colour does not
 // contrast with what is behind it (a shaded cell, a highlight, the dark or
 // light surface) is shown in a readable colour. The colour lives in a separate
 // sheet keyed by a `data-mw-vc` index that `serialize` and copy strip, so the
 // HTML that is sent or saved never carries it.
 function applyViewColors():void {
  viewTimer=undefined;
  if(disposed||!editor)return;
  const doc=editor.ownerDocument,view=doc.defaultView;if(!view)return;
  for(const shown of editor.querySelectorAll('[data-mw-vc]'))shown.removeAttribute('data-mw-vc');
  const surface=cssColorHex(view.getComputedStyle(doc.body).backgroundColor)??(dark()?'#1e1f22':'#ffffff');
  const rules:string[]=[],index=new Map<string,number>();
  const behind=(element:Element):string=>{for(let node:Element|null=element;node&&node!==doc.body;node=node.parentElement){const found=cssColorHex(view.getComputedStyle(node).backgroundColor);if(found)return found;}return surface;};
  const walker=doc.createTreeWalker(editor,4);let text:Node|null;const seen=new Set<Element>();
  while((text=walker.nextNode())){
   const element=text.parentElement;if(!element||seen.has(element)||!(text.textContent??'').trim())continue;
   seen.add(element);
   const color=cssColorHex(view.getComputedStyle(element).color);if(!color)continue;
   const background=behind(element);
   if(contrastRatio(color,background)>=UNREADABLE_BELOW)continue;
   const shown=readableColor(color,background);if(!shown)continue;
   let slot=index.get(shown);if(slot===undefined){slot=rules.length;index.set(shown,slot);rules.push(`[data-mw-vc="${slot}"]{color:${shown} !important}`);}
   element.setAttribute('data-mw-vc',String(slot));
  }
  if(!viewSheet){const Sheet=(view as Window&{CSSStyleSheet:typeof CSSStyleSheet}).CSSStyleSheet;viewSheet=new Sheet();doc.adoptedStyleSheets=[...doc.adoptedStyleSheets,viewSheet];}
  while(viewSheet.cssRules.length)viewSheet.deleteRule(0);rules.forEach(rule=>viewSheet!.insertRule(rule,viewSheet!.cssRules.length));
 }
 function scheduleViewColors():void {if(viewTimer===undefined)viewTimer=setTimeout(applyViewColors,120);}
 function installStyles(doc:Document):void {
  const view=doc.defaultView as (Window&{CSSStyleSheet:typeof CSSStyleSheet})|null;
  if(view){const host=frame.ownerDocument.defaultView?.getComputedStyle(frame),sheet=new view.CSSStyleSheet();sheet.replaceSync(`html,body{margin:0;min-height:100%;background:${host?.backgroundColor||'transparent'};color:${host?.color||'inherit'};color-scheme:${dark()?'dark':'light'}}html{height:100%}body{box-sizing:border-box;height:100%;padding:calc(.6rem - 1px) calc(.7rem - 1px);font:${HTML_MESSAGE_FONT}}#mw-native-editor{box-sizing:border-box;min-height:100%;outline:0}img{max-width:100%;height:auto}`+LIST_CSS+styles.join('\n').replace(/url\([^)]*\bcid:[^)]*\)/gi,'none')+(markers()?'#mw-native-editor :is(p,li,h1,h2,h3,h4,h5,h6)::after{content:"¶";opacity:.5}':''));doc.adoptedStyleSheets=viewSheet?[sheet,viewSheet]:[sheet];}
  scheduleViewColors();
 }
 function loadDocument(value=props.html):void {
  const doc=frame.contentDocument;if(!doc||!editor)return;
  const revision=++documentRevision;
  changed=false;
  commands.resetSelection();
  const body=nativeHtmlBody(stripEditorMarkers(value));dematerializeLists(body);
  styles=Array.from(body.querySelectorAll('style')).map(style=>style.textContent??'');
  body.querySelectorAll('style').forEach(style=>style.remove());
  const inlineStyles=Array.from(body.querySelectorAll<HTMLElement>('[style]')).map(element=>[element,element.getAttribute('style')??''] as const);
  const remembered=new Map<Element,Map<string,string>>();
  for(const element of body.querySelectorAll<HTMLElement>('*')){
   const original=new Map<string,string>();
   for(const name of ['src','background']){const value=element.getAttribute(name);if(value&&/^cid:/i.test(value)){original.set(name,value);element.removeAttribute(name);}}
   for(const name of Array.from(element.style)){const value=element.style.getPropertyValue(name);if(/url\([^)]*\bcid:/i.test(value))original.set(`style:${name==='background'?'background-image':name==='list-style'?'list-style-image':name}`,name==='background'?element.style.backgroundImage:name==='list-style'?element.style.listStyleImage:value);}
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
  setVersion(value=>value+1);
 }
 function insert(event:ClipboardEvent|DragEvent):void {
  event.preventDefault();const data='clipboardData'in event?event.clipboardData:event.dataTransfer;
  if(!data||!editor)return;
  const clipboardHtml=data.getData('text/html'),plain=data.getData('text/plain'),files=Array.from(data.files??[]);
  if(!clipboardHtml&&!plain&&!files.length)return;
  const doc=editor.ownerDocument,selection=doc.getSelection(),range=selection?.rangeCount?selection.getRangeAt(0).cloneRange():null,revision=documentRevision,edit=editRevision;
  pending=pending.catch(()=>undefined).then(async()=>{
   setFailed(false);
   const escape=(value:string)=>value.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
   if(props.htmlLimit&&clipboardHtml&&editorBodyBytes(clipboardHtml)>props.htmlLimit)throw new Error('HTML too large');
   let raw=clipboardHtml||`<p>${escape(plain).replace(/\n/g,'<br>')}</p>`;
   if(!clipboardHtml&&!plain&&files.length){
    if(files.some(file=>!['image/png','image/jpeg','image/gif','image/webp'].includes(file.type)))throw new Error('Unsupported image');
    if(props.upload){const images=await props.upload(files);if(!images.length)throw new Error('Image upload failed');raw=images.map(image=>`<img src="cid:${escape(image.cid)}" alt="${escape(image.alt)}">`).join('');}
    else if(props.embedded){if(props.htmlLimit&&files.reduce((bytes,file)=>bytes+Math.ceil(file.size/3)*4+editorBodyBytes(`<img src="data:${file.type};base64," alt="${escape(file.name)}">`),editorBodyBytes(serialize()))>props.htmlLimit)throw new Error('Image too large');const images=await Promise.all(files.map(file=>new Promise<string>((resolve,reject)=>{if(file.size>(props.htmlLimit??1024*1024)){reject(new Error('Image too large'));return;}const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=()=>reject(reader.error);reader.readAsDataURL(file);})));raw=images.map((src,index)=>`<img src="${escape(src)}" alt="${escape(files[index]!.name)}">`).join('');}
    else throw new Error('Image upload unavailable');
   }
   const safe=await normalizeHtmlSource(raw,props.owned(),props.embedded===true);
   if(disposed||revision!==documentRevision||!editor)return;
   const current=commands.range();
   if(edit!==editRevision||(range&&current&&(range.compareBoundaryPoints(0,current)!==0||range.compareBoundaryPoints(2,current)!==0))){setError(t('mail-editor-selection-changed'));return;}
   editor.focus();
   // Import allowlisted nodes without a child-realm HTML string sink. That
   // document intentionally has no Trusted Types policy or executable script.
   const parsed=nativeHtmlBody(safe.html);styles.push(...Array.from(parsed.querySelectorAll('style')).map(style=>style.textContent??''));parsed.querySelectorAll('style').forEach(style=>style.remove());
   const remembered=new Map<Element,Map<string,string>>();
   for(const element of parsed.querySelectorAll<HTMLElement>('*')){
    const original=new Map<string,string>();
    for(const name of ['src','background']){const value=element.getAttribute(name);if(value&&/^cid:/i.test(value)){original.set(name,value);element.removeAttribute(name);}}
    for(const name of Array.from(element.style)){const value=element.style.getPropertyValue(name);if(/url\([^)]*\bcid:/i.test(value)){original.set(`style:${name==='background'?'background-image':name==='list-style'?'list-style-image':name}`,name==='background'?element.style.backgroundImage:name==='list-style'?element.style.listStyleImage:value);element.style.setProperty(name,value.replace(/url\([^)]*\bcid:[^)]*\)/gi,'none'));}}
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
   installStyles(doc);previewImages(revision);commands.remember();emit();
  });
  void pending.catch(()=>{if(!disposed)setFailed(true);});
 }
 function imageFiles(files:File[]):void {commands.restore();insert({preventDefault(){},clipboardData:{files,getData:()=>''}} as unknown as ClipboardEvent);}
 async function performClipboard(action:'copy'|'cut'|'paste',mode:PasteMode='keep'):Promise<void> {
  const host=editor;if(!host)return;commands.restore();const revision=documentRevision,range=commands.range();
  try{
   if(action!=='paste'){if(!host.ownerDocument.execCommand(action))throw new Error('Clipboard unavailable');commands.remember();return;}
   const clipboard=frame.ownerDocument.defaultView?.navigator.clipboard;if(!clipboard)throw new Error('Clipboard unavailable');let html='',plain='';
   if(clipboard.read){const items=await clipboard.read();const item=items.find(item=>item.types.includes('text/html')||item.types.includes('text/plain'));if(!item)throw new Error('No text');const type=item.types.includes('text/html')?'text/html':'text/plain',blob=await item.getType(type);if(blob.size>(type==='text/html'?(props.htmlLimit??786432):786432))throw new Error('Too large');const value=await blob.text();if(type==='text/html')html=value;else plain=value;}else plain=await clipboard.readText();
   const current=commands.range();if(disposed||revision!==documentRevision||!range||!current||range.compareBoundaryPoints(0,current)!==0||range.compareBoundaryPoints(2,current)!==0){setError(t('mail-editor-selection-changed'));return;}
   if(plain.length>786432)throw new Error('Too large');commands.restore();{const merged=(()=>{if(mode!=='merge'||!html)return html;const parsed=new window.DOMParser().parseFromString(html,'text/html');mergeFormatting(parsed.body);return parsed.body.innerHTML;})();const text=plain||(html?new window.DOMParser().parseFromString(html,'text/html').body.textContent??'':'');insert({preventDefault(){},clipboardData:{files:[],getData:(type:string)=>type==='text/html'?(mode==='text'?'':merged):(mode==='text'?text:plain)}} as unknown as ClipboardEvent);};setError('');
  }catch(error){if(!disposed)setError(t('mail-editor-clipboard-unavailable'));if(action==='paste')throw error;}
 }
 function clipboard(action:'copy'|'cut'|'paste',mode:PasteMode='keep'):Promise<void>{const prior=pending,work=performClipboard(action,mode);if(action==='paste'){pending=Promise.all([prior,work]).then(()=>undefined);void pending.catch(()=>undefined);}return work.catch(()=>undefined);}
 function loaded():void {
  const doc=frame.contentDocument;if(!doc||disposed||doc.documentElement.getAttribute('data-mw-native-document')!=='true')return;
  if(editor?.ownerDocument===doc&&editor.isConnected)return;
  undoListeners();editor=doc.createElement('div');editor.id='mw-native-editor';editor.contentEditable='true';editor.tabIndex=0;editor.setAttribute('role','textbox');editor.setAttribute('aria-multiline','true');editor.setAttribute('aria-label',t('mail-compose-body'));doc.body.replaceChildren(editor);
  const input=()=>{commands.remember();emit();};const paste=(event:Event)=>insert(event as ClipboardEvent);const drop=(event:Event)=>insert(event as DragEvent);
  const before=(event:Event)=>{const type=(event as InputEvent).inputType;if(['insertFromPaste','insertFromDrop'].includes(type))event.preventDefault();if(type==='historyUndo'||type==='historyRedo'){event.preventDefault();moveHistory(type==='historyUndo'?-1:1);}};
  const key=(event:KeyboardEvent)=>{
   if(event.key==='Escape'&&painter.sticky()){painter.cancel();event.preventDefault();return;}
   if(stepOn()&&(isGrowKey(event)||isShrinkKey(event))){event.preventDefault();commands.remember();format.stepFont(isGrowKey(event)?1:-1);return;}
   if(clearOn()&&isClearKey(event)){event.preventDefault();commands.remember();format.clear('all');return;}
   if((event.ctrlKey||event.metaKey)&&!event.altKey&&(event.key==='='||event.key==='+')){const sup=event.key==='+';if(sup?supOn():subOn()){event.preventDefault();commands.remember();format.toggleEffect(sup?'superscript':'subscript');return;}}
   if(event.key==='Tab'&&!event.ctrlKey&&!event.metaKey&&!event.altKey&&indentOn()&&commands.current()?.closest('li')){event.preventDefault();commands.remember();format.indent(event.shiftKey?-1:1);return;}
   if(!(event.ctrlKey||event.metaKey)||event.altKey)return;if(event.key.toLowerCase()==='z'||event.key.toLowerCase()==='y'){event.preventDefault();moveHistory(event.key.toLowerCase()==='y'||event.shiftKey?1:-1);}};
  const selection=()=>commands.remember();
  const click=(event:Event)=>{const target=event.target as Element|null;
   const item=typeof target?.closest==='function'?target.closest('li[data-mw-checked]'):null;
   if(item&&item.parentElement?.getAttribute('data-mw-list')==='check'){const box=item.getBoundingClientRect(),x=(event as MouseEvent).clientX,rtl=doc.defaultView?.getComputedStyle(item).direction==='rtl';if(rtl?x>box.right:x<box.left){event.preventDefault();commands.remember();format.toggleChecked(item as HTMLElement);return;}}
   if(typeof target?.closest==='function'&&target.closest('a'))event.preventDefault();};
  const focus=(event:Event)=>{if(editor&&event.target&&editor.contains(event.target as Node)&&doc.activeElement!==editor)editor.focus({preventScroll:true});};
  const copy=(event:ClipboardEvent)=>{const selection=doc.getSelection();if(!editor||!event.clipboardData||!selection?.rangeCount||selection.isCollapsed)return;const range=selection.getRangeAt(0);if(!editor.contains(range.commonAncestorContainer))return;const inert=doc.implementation.createHTMLDocument(''),container=inert.createElement('div');container.append(inert.importNode(range.cloneContents(),true));container.querySelectorAll('[data-mw-vc]').forEach(shown=>shown.removeAttribute('data-mw-vc'));const sourceImages=Array.from(editor.querySelectorAll('img')).filter(image=>range.intersectsNode(image));Array.from(container.querySelectorAll('img')).forEach((image,index)=>{const source=sourceImages[index];const src=source?originals.get(source)?.get('src'):undefined;if(src)image.setAttribute('src',src);});event.clipboardData.setData('text/html',styles.map(style=>`<style>${style}</style>`).join('')+container.innerHTML);event.clipboardData.setData('text/plain',selection.toString());event.preventDefault();if(event.type==='cut'){doc.execCommand('delete');emit();}};
  doc.addEventListener('input',input);doc.addEventListener('paste',paste);doc.addEventListener('drop',drop);doc.addEventListener('beforeinput',before);doc.addEventListener('click',click);doc.addEventListener('pointerdown',focus);doc.addEventListener('copy',copy);doc.addEventListener('cut',copy);doc.addEventListener('selectionchange',selection);doc.addEventListener('keydown',key);const release=()=>{if(painter.sticky()&&!doc.getSelection()?.isCollapsed){commands.remember();painter.apply();}};doc.addEventListener('mouseup',release);
  undoListeners=()=>{doc.removeEventListener('mouseup',release);doc.removeEventListener('input',input);doc.removeEventListener('paste',paste);doc.removeEventListener('drop',drop);doc.removeEventListener('beforeinput',before);doc.removeEventListener('click',click);doc.removeEventListener('pointerdown',focus);doc.removeEventListener('copy',copy);doc.removeEventListener('cut',copy);doc.removeEventListener('selectionchange',selection);doc.removeEventListener('keydown',key);};
  loadDocument();history=[props.html];setHistoryIndex(0);setHistoryLength(1);if(props.bookmark)restoreBookmark(props.bookmark);setReady(true);finishReady();if(focusPending)focusEditor();
 }
 createEffect(()=>{version();if(editor)void ensureManropePreview(editor);});
 createEffect(on(()=>props.revision,()=>{if(ready()){const prior=bookmark();loadDocument();record(props.html);if(prior)restoreBookmark(prior);}},{defer:true}));
 createEffect(()=>{background();app?.theme();app?.appearance();markers();if(ready())queueMicrotask(()=>{if(!disposed&&editor)installStyles(editor.ownerDocument);});});
 onMount(()=>{const root=frame.ownerDocument.documentElement,observer=new MutationObserver(()=>{if(!disposed&&editor)installStyles(editor.ownerDocument);});observer.observe(root,{attributes:true});onCleanup(()=>observer.disconnect());});
 onMount(()=>{const editorApi:NativeHtmlVisualApi={async capture(){await admitted;let latest:Promise<void>;do{latest=pending;await latest;}while(latest!==pending);if(disposed)throw new DOMException('Editor closed','AbortError');return changed?serialize():stripEditorMarkers(props.html);},focus:focusEditor,refreshImages:()=>previewImages(documentRevision),undo:()=>moveHistory(-1),redo:()=>moveHistory(1),historyDepth:()=>({undo:historyIndex(),redo:historyLength()-historyIndex()-1}),bookmark};setApi({...editorApi,appendHtml:value=>{commands.restore();insert({preventDefault(){},clipboardData:{files:[],getData:(type:string)=>type==='text/html'?value:''}} as unknown as ClipboardEvent);},setHtml:value=>{pending=normalizeHtmlSource(value,props.owned(),props.embedded===true).then(safe=>{if(disposed)return;loadDocument(safe.html);emit();});void pending.catch(()=>setFailed(true));}});props.onReady(editorApi);});
 onCleanup(()=>{disposed=true;if(viewTimer!==undefined)clearTimeout(viewTimer);finishReady();undoListeners();});
 return <div class={css.visual}><NativeHtmlRibbon commands={commands} version={version} ready={ready} {...(props.ribbonSlots?{ribbonSlots:props.ribbonSlots}:{})} {...(props.externalHistory===undefined?{}:{externalHistory:props.externalHistory})} history={api} format={format} painter={painter} clipboard={clipboard} images={imageFiles} canImages={Boolean(props.upload||props.embedded)} report={setError} zoom={value=>{if(editor)editor.style.zoom=String(value/100);}} view={()=>({dark:dark(),markers:markers()})} toggleView={kind=>{if(kind==='dark')toggleBackground();else setMarkers(value=>!value);}}/><iframe ref={frame} class={css.frame} data-surface={background()} title={t('mail-compose-html-visual')} sandbox="allow-same-origin" referrerPolicy="no-referrer" srcdoc={FRAME_DOCUMENT} onLoad={loaded}/><span role="status">{error()|| (failed()?t('mail-compose-html-failed'):'')}</span></div>;
}
