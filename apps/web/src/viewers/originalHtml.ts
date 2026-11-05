// This optional display path preserves authored layout, not active content.
// Pure parsers run before any browser DOM parser can observe a resource URL.
import {parse,serialize,defaultTreeAdapter as tree,html,type DefaultTreeAdapterTypes as T} from 'parse5';
import {tokenize,TokenType as K,type CSSToken} from '@csstools/css-tokenizer';
import {contactAddressFromMailto} from '../modules/contacts/contactAddress.ts';
import {inspectMessageLink,type ReviewedMessage} from './messageLinks.ts';
import {signedSource,type ReaderReferences,type RemotePolicy,type SignedContent} from '../api/remote-content.ts';
import {TRACKER_HOSTS as trackers,BEACON_RULES as beacons,TRACKING_PARAMETERS as parameters,TRACKER_SUFFIXES,TRACKER_LABELS} from './originalHtmlPolicy.ts';

type Element=T.Element;
type Resource={id:string;value:string;kind:'remote'|'cid'|'data';tracker:boolean};
export interface OriginalHtml {html:string;refs:ReaderReferences;resources:Resource[]}
const EMPTY='data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
const attr=(node:Element,name:string)=>node.attrs.find(value=>value.name===name&&!value.namespace)?.value??null;
function set(node:Element,name:string,value:string):void{node.attrs=node.attrs.filter(item=>item.name!==name||item.namespace);node.attrs.push({name,value});}
const element=(name:string,attrs:Record<string,string>={})=>tree.createElement(name,html.NS.HTML,Object.entries(attrs).map(([name,value])=>({name,value})));
function walk(parent:T.ParentNode,visit:(node:Element)=>void):void{for(const node of parent.childNodes)if('tagName'in node){visit(node);walk(node,visit);}}
function http(raw:string):string|null{try{const url=new URL(raw.startsWith('//')?'https:'+raw:raw);return ['https:','http:'].includes(url.protocol)&&!url.username&&!url.password?url.href:null;}catch{return null;}}
function tracker(url:string):boolean{const parsed=new URL(url),path=parsed.pathname.toLowerCase(),keys=[...parsed.searchParams.keys()].map(value=>value.toLowerCase());return (TRACKER_LABELS as readonly string[]).includes(parsed.hostname.split('.')[0]??'')||[...trackers,...TRACKER_SUFFIXES].some(host=>parsed.hostname===host||parsed.hostname.endsWith('.'+host))||beacons.some(rule=>{const split=rule.indexOf(':'),kind=rule.slice(0,split),value=rule.slice(split+1);return kind==='path-end'?path.endsWith(value):kind==='segment'?path.split('/').includes(value):kind==='query'&&value.split(',').every(key=>keys.includes(key));});}
function cleanLink(value:string):string{const hash=value.indexOf('#'),end=hash<0?value.length:hash,query=value.indexOf('?');if(query<0||query>end)return value;const pairs=value.slice(query+1,end).split('&'),kept=pairs.filter(pair=>{let name=pair.split('=',1)[0]??'';try{name=decodeURIComponent(name);}catch{/* malformed keys are retained */}name=name.toLowerCase();return !parameters.some(key=>key.endsWith('*')?name.startsWith(key.slice(0,-1)):name===key);});return kept.length===pairs.length?value:value.slice(0,query)+(kept.length?'?'+kept.join('&'):'')+value.slice(end);}
function raster(value:string):boolean{const match=/^data:image\/(png|gif|jpeg|webp);base64,([a-z0-9+/=\r\n]+)$/i.exec(value);if(!match)return false;try{const bytes=atob(match[2]!.replace(/[\r\n]/g,'')).slice(0,16);return match[1]!.toLowerCase()==='png'?bytes.startsWith('\x89PNG\r\n\x1a\n'):match[1]!.toLowerCase()==='gif'?/^GIF8[79]a/.test(bytes):match[1]!.toLowerCase()==='jpeg'?bytes.startsWith('\xff\xd8\xff'):bytes.startsWith('RIFF')&&bytes.slice(8,12)==='WEBP';}catch{return false;}}
function endFunction(tokens:CSSToken[],start:number):number{let depth=1;for(let i=start+1;i<tokens.length;i++){const type=tokens[i]![0];if(type===K.Function||type===K.OpenParen)depth++;else if(type===K.CloseParen&&--depth===0)return i;}return tokens.length-1;}
const functions=new Set('calc min max clamp var env rgb rgba hsl hsla hwb lab lch oklab oklch color color-mix light-dark linear-gradient repeating-linear-gradient radial-gradient repeating-radial-gradient conic-gradient repeating-conic-gradient cubic-bezier steps translate translatex translatey translatez translate3d scale scalex scaley scalez scale3d rotate rotatex rotatey rotatez rotate3d skew skewx skewy matrix matrix3d perspective blur brightness contrast drop-shadow grayscale hue-rotate invert opacity saturate sepia counter counters not is where has nth-child nth-last-child nth-of-type nth-last-of-type lang dir host host-context slotted part selector style'.split(' '));
const atRules=new Set(['media','supports','layer','container','keyframes','-webkit-keyframes','page','property']);
function pixelAttribute(value:string|null):number|undefined{if(value===null||! /^(?:\d+(?:\.\d*)?|\.\d+)(?:px)?$/i.test(value.trim()))return undefined;const number=Number(value.trim().replace(/px$/i,''));return Number.isFinite(number)&&number>=0?number:undefined;}
function cssDimensions(source:string):{width?:number;height?:number}{
 const tokens=tokenize({css:source}),out:{width?:number;height?:number}={},important=new Set<string>();
 for(let start=0;start<tokens.length;){let end=start,depth=0;for(;end<tokens.length;end++){const kind=tokens[end]![0];if(kind===K.Function||kind===K.OpenParen)depth++;else if(kind===K.CloseParen)depth--;else if(kind===K.Semicolon&&depth===0)break;}
  const declaration=tokens.slice(start,end).filter(token=>token[0]!==K.Whitespace&&token[0]!==K.Comment&&token[0]!==K.EOF);start=end+1;
  const property=declaration[0];if(property?.[0]!==K.Ident||declaration[1]?.[0]!==K.Colon)continue;
  const name=property[4].value.toLowerCase();if(name!=='width'&&name!=='height')continue;
  const bang=declaration[declaration.length-2],last=declaration[declaration.length-1];const priority=bang?.[0]===K.Delim&&bang[4].value==='!'&&last?.[0]===K.Ident&&last[4].value.toLowerCase()==='important';if(important.has(name)&&!priority)continue;
  const values=declaration.slice(2,priority?-2:undefined),value=values[0];let dimension:number|undefined;
  if(values.length===1&&value&&(value[0]===K.Number||value[0]===K.Dimension&&value[4].unit.toLowerCase()==='px')&&Number.isFinite(value[4].value)&&value[4].value>=0)dimension=value[4].value;
  if(dimension===undefined)delete out[name];else out[name]=dimension;if(priority)important.add(name);
 }
 return out;
}
/** Decoded CSS tokens prevent escaped url/import/function names hiding requests.
 * Unsupported resource functions and at-rules are removed conservatively. */
function css(source:string,resource:(value:string)=>string,allowUrls=true):string{
 const tokens=tokenize({css:source});let output='';
 for(let i=0;i<tokens.length;i++){
  const token=tokens[i]!;
  if(token[0]===K.AtKeyword&&!atRules.has(token[4].value.toLowerCase())){
   let depth=0;
   for(i++;i<tokens.length;i++){const type=tokens[i]![0];if(type===K.OpenCurly)depth++;else if(type===K.CloseCurly){if(--depth<=0)break;}else if(type===K.Semicolon&&depth===0)break;}
   continue;
  }
  if(token[0]===K.URL){output+=allowUrls?'url('+JSON.stringify(resource(token[4].value))+')':'none';continue;}
  if(token[0]===K.Function){
   const name=token[4].value.toLowerCase();
   if(name==='url'){
    const end=endFunction(tokens,i),args=tokens.slice(i+1,end).filter(value=>value[0]!==K.Whitespace&&value[0]!==K.Comment);
    output+=allowUrls?'url('+JSON.stringify(args.length===1&&args[0]![0]===K.String?resource(args[0]![4].value):EMPTY)+')':'none';i=end;continue;
   }
   if(!functions.has(name)){i=endFunction(tokens,i);output+='none';continue;}
  }
  if(token[0]===K.BadURL||token[0]===K.BadString){output+='none';continue;}
  output+=token[1];
 }
 return output;
}
const DROP=new Set('script iframe frame frameset object embed applet portal fencedframe form input button select option textarea audio video source track link meta base template noscript animate animatetransform animatemotion set mpath use feimage'.split(' '));
const REMOVE=new Set('src srcset imagesrcset href background poster manifest ping action formaction form target download integrity nonce is autofocus crossorigin lowsrc dynsrc codebase archive data profile longdesc srcdoc'.split(' '));

/** No DOM APIs are used here. Even hostile full documents are inert input. */
export function prepareOriginalHtml(source:string,policy:{trackers:boolean;cleanLinks:boolean}):OriginalHtml{
 if(source.length>8*1024*1024)throw new Error('Original HTML is too large');
 const doc=parse(source,{scriptingEnabled:true}),resources:Resource[]=[],urls=new Set<string>(),cids=new Set<string>(),links=new Set<string>();
 const markers:Element[]=[];let visits=0;
 const resource=(raw:string,declaredTracker=false):string=>{
  const value=raw.trim();if(/^#[a-z0-9_-]+$/i.test(value))return value;
  let kind:Resource['kind'],canonical:string;
  if(/^cid:/i.test(value)){try{canonical=decodeURIComponent(value.slice(4)).replace(/^<|>$/g,'');}catch{return EMPTY;}if(!canonical||canonical.length>512)return EMPTY;kind='cid';cids.add(canonical);}
  else if(raster(value)){kind='data';canonical=value;}
  else{const url=http(value);if(!url)return EMPTY;kind='remote';canonical=url;}
  if(resources.length>=1024)return EMPTY;
  const tracking=kind==='remote'&&(declaredTracker||tracker(canonical));
  const id=String(resources.length);resources.push({id,value:canonical,kind,tracker:tracking});
  if(kind==='remote'){
   if(!(policy.trackers&&tracking)&&urls.size<256)urls.add(canonical);
   markers.push(element('span',{hidden:'','data-mw-rc':id,'data-mw-rc-url':canonical,'data-mw-blocked-host':new URL(canonical).hostname,...(tracking?{'data-mw-tracker':''}:{})}));
  }
  return 'mw-original:'+id;
 };
 const scrub=(parent:T.ParentNode):void=>{
  parent.childNodes=parent.childNodes.filter(node=>node.nodeName!=='#comment'&&(!('tagName'in node)||!DROP.has(node.tagName.toLowerCase())));
  for(const node of parent.childNodes){
   if(!('tagName'in node))continue;
   if(++visits>100000)throw new Error('Original HTML has too many elements');
   const name=node.tagName.toLowerCase(),src=attr(node,'src'),background=attr(node,'background'),href=attr(node,'href'),style=attr(node,'style');
   const dimensions=cssDimensions(style??''),width=dimensions.width??pixelAttribute(attr(node,'width')),height=dimensions.height??pixelAttribute(attr(node,'height'));
   const tiny=width!==undefined&&height!==undefined&&width<=2&&height<=2;
   node.attrs=node.attrs.filter(value=>!value.namespace&&!value.name.startsWith('on')&&!value.name.startsWith('data-mw-')&&!REMOVE.has(value.name)&&value.name!=='style');
   if(style!==null)set(node,'style',css(style,value=>resource(value)));
   if(name==='img'&&src!==null){set(node,'src',resource(src,tiny));set(node,'loading','eager');set(node,'decoding','async');set(node,'referrerpolicy','no-referrer');}
   if(background!==null)set(node,'background',resource(background));
   if(href!==null&&(name==='a'||name==='area')&&node.namespaceURI===html.NS.HTML){const validated=inspectMessageLink(href,'');if(validated){const value=policy.cleanLinks&&validated.kind==='web'?cleanLink(validated.href):validated.href;set(node,'href',value);if(validated.kind==='web'&&links.size<256)links.add(value);}else if(/^#[a-z0-9_-]+$/i.test(href))set(node,'href',href);}
   if(name==='style'){for(const text of node.childNodes)if(tree.isTextNode(text))text.value=css(text.value,value=>resource(value));}
   scrub(node);
  }
 };
 scrub(doc);
 const body=doc.childNodes.find((node):node is Element=>'tagName'in node&&node.tagName==='html')?.childNodes.find((node):node is Element=>'tagName'in node&&node.tagName==='body');
 if(!body)throw new Error('Original HTML has no body');
 for(const marker of markers)tree.appendChild(body,marker);
 return {html:serialize(doc),resources,refs:{urls:[...urls],cids:[...cids].slice(0,256),trackers:resources.filter(value=>value.tracker).length,files:[...links].filter(value=>/\.(?:pdf|zip|gz|tar|7z|docx?|xlsx?|pptx?|txt|csv|rtf|odt|ods|epub|ics)$/i.test(new URL(value).pathname)).slice(0,32),links:[...links]}};
}

/** Render only signed resources returned by this server; original view always
 * proxies images, even if normal sanitized mail uses a direct-image preference. */
export function renderOriginalHtml(prepared:OriginalHtml,signed:SignedContent,policy:RemotePolicy,cidParts:Readonly<Record<string,string>>,serverBase:string,media=true):string{
 const doc=parse(prepared.html),images=new Map(signed.images.map(value=>[value.url,signedSource(value.src,'i',serverBase)])),parts=new Map(signed.parts.map(value=>[value.blobId,signedSource(value.src,'p',serverBase)]));
 const sources=new Map(prepared.resources.map(value=>[value.id,!media||policy.trackers&&value.tracker?EMPTY:value.kind==='data'?value.value:value.kind==='cid'?(parts.get(cidParts[value.value]??'')??EMPTY):policy.mode==='block'?EMPTY:(images.get(value.value)??EMPTY)]));
 const replace=(value:string)=>value.replace(/mw-original:(\d+)/g,(_,id:string)=>sources.get(id)??EMPTY);
 walk(doc,node=>{if(!media){node.childNodes=node.childNodes.filter(child=>!('tagName'in child)||!['img','picture'].includes(child.tagName));node.attrs=node.attrs.filter(value=>value.name!=='background');}for(const value of node.attrs)if(['src','background','style'].includes(value.name))value.value=!media&&value.name==='style'?css(value.value,()=>EMPTY,false):replace(value.value);if(node.tagName==='style')for(const text of node.childNodes)if(tree.isTextNode(text))text.value=media?replace(text.value):css(text.value,()=>EMPTY,false);});
 const head=doc.childNodes.find((node):node is Element=>'tagName'in node&&node.tagName==='html')?.childNodes.find((node):node is Element=>'tagName'in node&&node.tagName==='head');
 if(!head)throw new Error('Original HTML has no head');
 let origins='';try{const base=new URL(serverBase);if(['https:','http:'].includes(base.protocol)&&!base.username&&!base.password){const prefix=base.origin+base.pathname.replace(/\/$/,'')+'/api/rc/';origins=` ${prefix}i/ ${prefix}p/`;}}catch{/* no signed resources admitted */}
 const prepend=(node:Element):void=>{const first=head.childNodes[0];if(first)tree.insertBefore(head,node,first);else tree.appendChild(head,node);};
 prepend(element('meta',{'http-equiv':'Content-Security-Policy',content:`default-src 'none'; script-src 'none'; script-src-attr 'none'; style-src 'unsafe-inline'; img-src ${media?'data:'+origins:"'none'"}; font-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'`}));
 prepend(element('meta',{name:'referrer',content:'no-referrer'}));
 return '<!doctype html>'+serialize(doc).replace(/^<!DOCTYPE html>/i,'');
}

/** Link review shares the existing destination validator but stays in the pure
 * parser so a full original document never gets parsed into the app's DOM. */
export function reviewOriginalLinks(source:string,help:string,enabled=false):ReviewedMessage{
 const doc=parse(source),links:ReviewedMessage['links']=[],seen=new Set<string>();let truncated=false;
 walk(doc,node=>{if(!['a','area'].includes(node.tagName))return;const href=attr(node,'href');if(!href||href.startsWith('#'))return;const label=(node.childNodes.map(value=>tree.isTextNode(value)?value.value:'').join('')).trim();const link=inspectMessageLink(href,label);if(link&&!seen.has(link.href)){seen.add(link.href);if(links.length<100)links.push(link);else truncated=true;}node.attrs=node.attrs.filter(value=>!['href','target','ping','download','data-mw-contact-address'].includes(value.name));const contact=contactAddressFromMailto(href);if(contact)set(node,'data-mw-contact-address',contact);if(enabled&&link){set(node,'href',link.href);set(node,'target','_blank');set(node,'rel','noopener noreferrer');set(node,'referrerpolicy','no-referrer');}else{set(node,'aria-disabled','true');set(node,'title',help);}});
 return {html:serialize(doc),links,truncated};
}

export function originalHtmlReport(prepared:OriginalHtml):{blockedHosts:string[];blockedCount:number;trackerCount:number}{const remote=prepared.resources.filter(value=>value.kind==='remote');return {blockedHosts:[...new Set(remote.map(value=>new URL(value.value).hostname))].sort(),blockedCount:remote.length,trackerCount:remote.filter(value=>value.tracker).length};}

/** Retry only previously approved failed sources without flattening document
 * root styling or reparsing authored HTML into the application DOM. */
export function replaceOriginalImages(source:string,images:SignedContent['images'],failedRefs:readonly string[],replacements:SignedContent['images'],base:string):{html:string;imageSources:SignedContent['images']}{
 const failed=new Set(failedRefs),changes=new Map<string,string>();
 const next=images.map(image=>{if(!image.ref||!failed.has(image.ref))return image;const replacement=replacements.find(value=>value.url===image.url),before=signedSource(image.src,'i',base),after=replacement?signedSource(replacement.src,'i',base):null;if(!replacement||!before||!after)return image;changes.set(before,after);return replacement;});
 const doc=parse(source),replace=(value:string)=>{for(const [before,after]of changes)value=value.replaceAll(before,after);return value;};walk(doc,node=>{for(const item of node.attrs)if(['src','background','style'].includes(item.name))item.value=replace(item.value);if(node.tagName==='style')for(const text of node.childNodes)if(tree.isTextNode(text))text.value=replace(text.value);});
 return {html:serialize(doc),imageSources:next};
}
