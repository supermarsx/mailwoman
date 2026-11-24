import {inertEmailDocument,setInertAttribute,type InertEmailDocument} from './inertEmailDocument.ts';
import {listMailDraft} from '../api/mailingList.ts';
import {normalizeContactAddress,contactAddressFromMailto} from '../modules/contacts/contactAddress.ts';
export interface MessageLink {href:string;label:string;host:string;kind:'web'|'mail';warnings:('http'|'address'|'international'|'mismatch'|'direction')[]}
export interface ReviewedMessage {html:string;links:MessageLink[];truncated:boolean}
const control=(value:string):boolean=>[...value].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127);
const direction=/[\u202a-\u202e\u2066-\u2069]/u;
export function inspectMessageLink(raw:string,label:string):MessageLink|null{
 if(!raw||raw.length>4096||control(raw)||direction.test(raw))return null;
 // Authored query values often contain literal spaces. Let the URL parser
 // encode them instead of discarding an otherwise valid destination.
 const destination=raw.trim();if(!destination)return null;
 let url:URL;try{url=new URL(destination.startsWith('//')?'https:'+destination:destination);}catch{return null;}
 if(url.username||url.password)return null;
 if(url.protocol==='mailto:'){
  const draft=listMailDraft({href:url.href,kind:'mail',host:''});return draft?{href:url.href,label:label.slice(0,256),host:draft.to,kind:'mail',warnings:[]}:null;
 }
 if(!['https:','http:'].includes(url.protocol))return null;
 const warnings:MessageLink['warnings']=[];
 if(url.protocol==='http:')warnings.push('http');
 if(url.hostname.startsWith('[')||/^\d+\.\d+\.\d+\.\d+$/.test(url.hostname))warnings.push('address');
 if(url.hostname.split('.').some(part=>part.startsWith('xn--')))warnings.push('international');
 if(direction.test(label))warnings.push('direction');
 try{const shown=new URL(label.trim());if(['http:','https:'].includes(shown.protocol)&&shown.hostname!==url.hostname)warnings.push('mismatch');}catch{/* Ordinary labels are not claimed hostnames. */}
 return {href:url.href,label:label.slice(0,256),host:url.host,kind:'web',warnings};
}
/** Linkify only text nodes in the display copy; never attributes or authored links. */
function linkifyAddresses(doc:InertEmailDocument):void{
 const owner=doc.body.ownerDocument;
 const walker=owner.createTreeWalker(doc.body,NodeFilter.SHOW_TEXT);const nodes:Text[]=[];let next:Node|null;
 let visited=0;
 while(visited++<10000&&(next=walker.nextNode())){const node=next as Text;if(node.textContent?.includes('@')&&node.textContent.length<=65536&&!node.parentElement?.closest('a,button,script,style,textarea,svg,math,code'))nodes.push(node);}
 let remaining=500;
 for(const node of nodes){if(!remaining)break;const value=node.data;const matches=[...value.matchAll(/(?<![\p{L}\p{N}.!#$%&'*+/=?^_`{|}~-])[\p{L}\p{N}.!#$%&'*+/=?^_`{|}~-]{1,64}@[\p{L}\p{N}-]{1,63}(?:\.[\p{L}\p{N}-]{1,63}){1,8}(?![\p{L}\p{N}-]|\.[\p{L}\p{N}-])/giu)].slice(0,remaining).filter(match=>!!normalizeContactAddress(match[0])&&!/(?:https?:|www\.)/i.test(value.slice(0,match.index).match(/\S*$/)?.[0]??''));if(!matches.length)continue;
 const fragment=owner.createDocumentFragment();let offset=0;
 for(const match of matches){if(!remaining)break;const address=normalizeContactAddress(match[0])!;fragment.append(owner.createTextNode(value.slice(offset,match.index)));const a=owner.createElement('a');a.textContent=match[0];a.setAttribute('href',`mailto:${address}`);fragment.append(a);offset=match.index!+match[0].length;remaining--;}
 fragment.append(owner.createTextNode(value.slice(offset)));node.replaceWith(fragment);
 }
}
/** Only display copies enter here. Source MIME, quotes and exports stay intact.
 * Body navigation is inert until explicit consent for this message. Only
 * validated destinations can then open a new context without an opener/referrer. */
export function reviewMessageLinks(html:string,help:string,enabled=false):ReviewedMessage{
 const doc=inertEmailDocument(html);linkifyAddresses(doc);const links:MessageLink[]=[];const seen=new Set<string>();let truncated=false;
 for(const anchor of doc.querySelectorAll('a,area')){
  anchor.removeAttribute('data-mw-contact-address');
  const raw=anchor.getAttribute('href');
  const contact=raw?contactAddressFromMailto(raw):null;
  if(contact)anchor.setAttribute('data-mw-contact-address',contact);
  if(raw&&!raw.startsWith('#')){
   const link=inspectMessageLink(raw,anchor.textContent?.trim()??'');
   if(link&&!seen.has(link.href)){seen.add(link.href);if(links.length<100)links.push(link);else truncated=true;}
   if(enabled&&link){anchor.setAttribute('href',link.href);anchor.removeAttribute('aria-disabled');anchor.removeAttribute('title');anchor.setAttribute('target','_blank');anchor.setAttribute('rel','noopener noreferrer');anchor.setAttribute('referrerpolicy','no-referrer');}
   else{anchor.removeAttribute('href');anchor.setAttribute('aria-disabled','true');anchor.setAttribute('title',help);}
  }
  if(contact){anchor.removeAttribute('aria-disabled');anchor.setAttribute('role','link');anchor.setAttribute('tabindex','0');anchor.setAttribute('title',contact);setInertAttribute(anchor,'style',`${(anchor.getAttribute('style')??'').trim().replace(/;?$/,';').replace(/^;$/,'')}cursor:pointer;text-decoration:underline`);}
  if(!enabled||!anchor.hasAttribute('href')||raw?.startsWith('#'))anchor.removeAttribute('target');anchor.removeAttribute('ping');anchor.removeAttribute('download');
 }
 return {html:'<!doctype html>'+doc.documentElement.outerHTML,links,truncated};
}
