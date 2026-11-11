import {parse,serialize,defaultTreeAdapter as tree,type DefaultTreeAdapterTypes as T} from 'parse5';
import {tokenize,TokenType as K} from '@csstools/css-tokenizer';
import {apiFetch} from '../../api/http.ts';
import {signContent,signedSource,remoteServerBase,type SignedContent} from '../../api/remote-content.ts';
import type {StoredInlineImage} from './drafts-store.ts';
import {embeddedImageFile} from './inlineImages.ts';

const MAX_IMAGES=64,MAX_BYTES=16*1024*1024,MAX_TOTAL=32*1024*1024,DEFAULT_IMAGE_BUDGET=90_000,MAX_PREPARATION_BUDGET=300_000;
type Element=T.Element;
type Image={key:string;kind:'remote'|'part'|'data';value:string};
const attr=(node:Element,name:string)=>node.attrs.find(item=>item.name===name)?.value??'';
const abort=()=>new DOMException('Reply preparation cancelled','AbortError');
function bounded<T>(work:Promise<T>,signal:AbortSignal):Promise<T>{
 if(signal.aborted)return Promise.reject(abort());
 return new Promise((resolve,reject)=>{const cancelled=()=>reject(abort());signal.addEventListener('abort',cancelled,{once:true});work.then(resolve,reject).finally(()=>signal.removeEventListener('abort',cancelled));});
}
function cssUrls(source:string,replace:(value:string)=>string):string {
 const tokens=tokenize({css:source});let result='';
 for(let index=0;index<tokens.length;index++){
  const token=tokens[index]!;
  if(token[0]===K.URL){const value=replace(token[4].value);result+=value?'url('+JSON.stringify(value)+')':'none';continue;}
  if(token[0]===K.Function&&token[4].value.toLowerCase()==='url'){
   index++;while(index<tokens.length&&(tokens[index]![0]===K.Whitespace||tokens[index]![0]===K.Comment))index++;
   const argument=tokens[index];while(index<tokens.length&&tokens[index]![0]!==K.CloseParen)index++;
   const value=argument?.[0]===K.String?replace(argument[4].value):'';result+=value?'url('+JSON.stringify(value)+')':'none';continue;
  }
  result+=token[1];
 }
 return result;
}
export interface ReplyImageOptions {
 emailId:string;accountId:string;cidParts:Readonly<Record<string,string>>;
 allowRemote:boolean;allowInline:boolean;blockTrackers:boolean;
 imageLoadBudgetMs?:number|undefined;
 current:()=>boolean;signal?:AbortSignal;
 upload:(file:File,signal:AbortSignal)=>Promise<StoredInlineImage>;
 sign?:typeof signContent;fetcher?:typeof apiFetch;
}

/** Input is the current reader's sanitized quote, never sender HTML or the
 * rewritten display document. Only freshly authorized app endpoints are read;
 * every outgoing resource becomes an owned CID or is removed. */
export async function embedReplyImages(html:string,options:ReplyImageOptions):Promise<{html:string;inline:StoredInlineImage[];omitted:number}> {
 const doc=parse(html),images=new Map<string,Image>(),markers=new Map<string,{url:string;tracker:boolean}>();
 const each=(parent:T.ParentNode,visit:(node:Element)=>void):void=>{for(const node of parent.childNodes)if('tagName'in node){visit(node);each(node,visit);}};
 each(doc,node=>{if(attr(node,'data-mw-rc'))markers.set(attr(node,'data-mw-rc'),{url:attr(node,'data-mw-rc-url'),tracker:node.attrs.some(item=>item.name==='data-mw-tracker')});});
 const denied=new Set<string>();
 const resource=(raw:string):string=>{
  const value=raw.trim();if(value.startsWith('#'))return value;
  let image:Image|undefined;
  const marker=/^mw-rc:(\d+)$/.exec(value),reference=marker?markers.get(marker[1]!):undefined;
  if(reference){if(options.allowRemote&&!(options.blockTrackers&&reference.tracker))image={key:'url:'+reference.url,kind:'remote',value:reference.url};}
  else if(/^cid:/i.test(value)){let cid='';try{cid=decodeURIComponent(value.slice(4)).replace(/^<|>$/g,'');}catch{/* refused */}const blob=options.cidParts[cid];if(options.allowInline&&blob)image={key:'part:'+blob,kind:'part',value:blob};}
  else if(options.allowInline&&/^data:image\/(png|jpeg|gif|webp);base64,/i.test(value))image={key:value,kind:'data',value};
  if(!image||(!images.has(image.key)&&images.size>=MAX_IMAGES)){if(value)denied.add(value);return '';}
  images.set(image.key,image);return 'mw-reply:'+encodeURIComponent(image.key);
 };
 each(doc,node=>{
  node.attrs=node.attrs.filter(item=>!item.name.startsWith('data-mw-')&&item.name!=='srcset');
  for(const item of node.attrs){if(['src','background'].includes(item.name))item.value=resource(item.value);else if(item.name==='style')item.value=cssUrls(item.value,resource);}
  if(node.tagName==='style')for(const text of node.childNodes)if(tree.isTextNode(text))text.value=cssUrls(text.value,resource);
 });
 const controller=new AbortController(),signal=controller.signal;
 const cancel=()=>controller.abort();options.signal?.addEventListener('abort',cancel,{once:true});if(options.signal?.aborted)cancel();
 const configured=options.imageLoadBudgetMs;
 const imageBudget=typeof configured==='number'&&Number.isFinite(configured)?Math.max(1_000,Math.min(300_000,configured)):DEFAULT_IMAGE_BUDGET;
 const deadline=Math.min(MAX_PREPARATION_BUDGET,imageBudget*Math.max(1,Math.ceil(images.size/2)));
 const timer=setTimeout(cancel,deadline),inline:StoredInlineImage[]=[],resolved=new Map<string,string>();let bytesRead=0,budgetExhausted=false;
 const current=()=>{if(!options.current())controller.abort();return !signal.aborted;};
 try{
  if(!options.current())throw abort();
  let signed:SignedContent={images:[],parts:[],files:[],refused:[]};
  const candidates=[...images.values()];
  if(candidates.some(image=>image.kind!=='data')){
   try{signed=await bounded((options.sign??signContent)(options.emailId,{urls:candidates.filter(image=>image.kind==='remote').map(image=>image.value),parts:candidates.filter(image=>image.kind==='part').map(image=>image.value)},options.accountId),signal);}catch{/* quote text remains available */}
  }
  let index=0;
  const worker=async()=>{
   while(index<candidates.length&&!budgetExhausted&&current()){
    const image=candidates[index++]!;
    const imageController=new AbortController();
    const cancelImage=()=>imageController.abort();
    signal.addEventListener('abort',cancelImage,{once:true});
    if(signal.aborted)cancelImage();
    const imageSignal=imageController.signal;
    const imageTimer=setTimeout(()=>imageController.abort(),imageBudget);
    try{
     let file:File|null=null;
     if(image.kind==='data'){file=embeddedImageFile(image.value,'reply-image.png');bytesRead+=file?.size??0;if(bytesRead>MAX_TOTAL){budgetExhausted=true;break;}}
     else{
      const path=image.kind==='remote'?signed.images.find(item=>item.url===image.value)?.src:signed.parts.find(item=>item.blobId===image.value)?.src;
      const src=path?signedSource(path,image.kind==='remote'?'i':'p',remoteServerBase()):null;
      if(!src)continue;
      const response=await bounded((options.fetcher??apiFetch)(src,{signal:imageSignal,redirect:'error',headers:{'X-Mailwoman-Account':options.accountId}}),imageSignal);
      if(!current()||budgetExhausted){void response.body?.cancel().catch(()=>undefined);break;}
      if(!response.ok||response.headers.get('x-mw-remote-content')==='tracker'||response.headers.get('content-type')?.split(';')[0]?.trim().toLowerCase()!=='image/png'||Number(response.headers.get('content-length'))>MAX_BYTES){void response.body?.cancel().catch(()=>undefined);continue;}
      const reader=response.body?.getReader();if(!reader)continue;const chunks:Uint8Array[]=[];let length=0;
      try{for(;;){if(budgetExhausted||!current())throw abort();const chunk=await bounded(reader.read(),imageSignal);if(chunk.done)break;length+=chunk.value.length;bytesRead+=chunk.value.length;if(bytesRead>MAX_TOTAL)budgetExhausted=true;if(length>MAX_BYTES||budgetExhausted)throw new Error('Image byte limit');chunks.push(chunk.value);}}finally{void reader.cancel().catch(()=>undefined);}
      const data=new Uint8Array(length);let offset=0;for(const chunk of chunks){data.set(chunk,offset);offset+=chunk.length;}
      if(data.length<24||!data.slice(0,8).every((byte,index)=>byte===[137,80,78,71,13,10,26,10][index]))continue;
      const view=new DataView(data.buffer),width=view.getUint32(16),height=view.getUint32(20);if(width===0||height===0||width>16384||height>16384||width*height*4>256*1024*1024||options.blockTrackers&&width<=2&&height<=2)continue;
      file=new File([data],'reply-image.png',{type:'image/png'});
     }
     if(!file||file.size>MAX_BYTES||!current()||imageSignal.aborted)continue;
     const stored=await bounded(options.upload(file,imageSignal),imageSignal);
     if(!current())break;
     if(!/^[A-Za-z0-9!#$%&'*+/=?^_`{|}~.@-]{1,255}$/.test(stored.cid)||!stored.blobId)continue;
     inline.push(stored);resolved.set(image.key,'cid:'+stored.cid);
    }catch{/* One failure must not erase the reply or its other images. */}
    finally{clearTimeout(imageTimer);signal.removeEventListener('abort',cancelImage);}
   }
  };
  await Promise.all([worker(),worker()]);
  if(!options.current()||options.signal?.aborted)throw abort();
  const replace=(value:string)=>{if(value.startsWith('#'))return value;try{return value.startsWith('mw-reply:')?resolved.get(decodeURIComponent(value.slice(9)))??'':'';}catch{return '';}};
  each(doc,node=>{for(const item of node.attrs){if(['src','background'].includes(item.name))item.value=replace(item.value);else if(item.name==='style')item.value=cssUrls(item.value,replace);}node.attrs=node.attrs.filter(item=>!['src','background'].includes(item.name)||item.value!=='');if(node.tagName==='style')for(const text of node.childNodes)if(tree.isTextNode(text))text.value=cssUrls(text.value,replace);});
  return {html:serialize(doc),inline,omitted:denied.size+images.size-resolved.size};
 }finally{clearTimeout(timer);options.signal?.removeEventListener('abort',cancel);}
}
