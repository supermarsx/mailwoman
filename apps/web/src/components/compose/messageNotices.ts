import {parseFragment,serialize,defaultTreeAdapter as tree,type DefaultTreeAdapterTypes as T} from 'parse5';
export type NoticeKind='new'|'reply'|'forward';
export interface MessageNotice {
 id:string;title:string;text:string;html:string;enabled:boolean;automatic:boolean;
 identities:string[];kinds:NoticeKind[];audience:'all'|'internal'|'external';
 domains:string[];subjectContains:string;bodyContains:string;
}
export interface NoticeChoice {mode:'auto'|'none'|'manual';ids:string[]}
export interface NoticeContext {identityId:string;sender:string;recipients:readonly string[];kind:NoticeKind;subject:string;body:string;internalDomains?:readonly string[]}
const strings=(value:unknown,max:number,length:number):value is string[]=>Array.isArray(value)&&value.length<=max&&value.every(item=>typeof item==='string'&&item.length>0&&item.length<=length&&![...item].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127));
export function validMessageNotices(value:unknown):value is MessageNotice[]{
 if(!Array.isArray(value)||value.length>20||new TextEncoder().encode(JSON.stringify(value)).length>262144)return false;
 const seen=new Set<string>();
 return value.every(row=>{
  if(!row||typeof row!=='object'||Object.keys(row).some(key=>!['id','title','text','html','enabled','automatic','identities','kinds','audience','domains','subjectContains','bodyContains'].includes(key)))return false;
  if(typeof row.id!=='string'||!/^[A-Za-z0-9_-]{1,80}$/.test(row.id)||seen.has(row.id))return false;seen.add(row.id);
  return typeof row.title==='string'&&row.title.trim().length>0&&row.title.length<=80&&typeof row.text==='string'&&row.text.trim().length>0&&row.text.length<=8000&&typeof row.html==='string'&&row.html.length<=32000&&typeof row.enabled==='boolean'&&typeof row.automatic==='boolean'&&strings(row.identities,50,512)&&strings(row.kinds,3,10)&&row.kinds.length>0&&row.kinds.every((kind:string)=>['new','reply','forward'].includes(kind))&&['all','internal','external'].includes(row.audience)&&strings(row.domains,50,253)&&row.domains.every((domain:string)=>/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/.test(domain))&&typeof row.subjectContains==='string'&&row.subjectContains.length<=200&&typeof row.bodyContains==='string'&&row.bodyContains.length<=500;
 });
}
export function matchingNotices(rows:readonly MessageNotice[],context:NoticeContext,choice:NoticeChoice={mode:'auto',ids:[]}):MessageNotice[]{
 if(choice.mode==='none')return [];
 const domain=(address:string)=>address.trim().toLowerCase().split('@')[1]??'';
 const internal=new Set([domain(context.sender),...(context.internalDomains??[]).map(value=>value.toLowerCase())].filter(Boolean));
 const recipientDomains=context.recipients.map(domain).filter(Boolean);
 const outside=recipientDomains.some(value=>!internal.has(value));
 return rows.filter(row=>{
  if(!row.enabled)return false;
  if(choice.mode==='manual')return choice.ids.includes(row.id);
  return row.automatic&&(!row.identities.length||row.identities.includes(context.identityId)||row.identities.some(value=>value.toLowerCase()===context.sender.toLowerCase()))&&row.kinds.includes(context.kind)&&
   (row.audience==='all'||recipientDomains.length>0&&(row.audience==='external'?outside:!outside))&&
   (!row.domains.length||recipientDomains.some(value=>row.domains.includes(value)))&&
   (!row.subjectContains.trim()||context.subject.toLocaleLowerCase().includes(row.subjectContains.trim().toLocaleLowerCase()))&&
   (!row.bodyContains.trim()||context.body.toLocaleLowerCase().includes(row.bodyContains.trim().toLocaleLowerCase()));
 });
}

/** Place notices immediately after a managed signature, retaining a following quote. */
export function appendNoticeHtml(html:string,notices:string):string {
 if(!notices)return html;
 const root=parseFragment(html),extra=parseFragment(notices);
 let signature:T.Element|undefined;
 const visit=(parent:T.ParentNode)=>{for(const child of parent.childNodes)if(tree.isElementNode(child)){if(child.attrs.some(attr=>attr.name==='data-mw-signature'))signature=child;visit(child);}};
 visit(root);
 if(!signature?.parentNode)return html+notices;
 const parent=signature.parentNode,index=parent.childNodes.indexOf(signature);
 for(const child of extra.childNodes)child.parentNode=parent;
 parent.childNodes.splice(index+1,0,...extra.childNodes);
 return serialize(root);
}
