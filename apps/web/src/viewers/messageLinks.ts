import {listMailDraft} from '../api/mailingList.ts';
export interface MessageLink {href:string;label:string;host:string;kind:'web'|'mail';warnings:('http'|'address'|'international'|'mismatch'|'direction')[]}
export interface ReviewedMessage {html:string;links:MessageLink[];truncated:boolean}
const control=(value:string):boolean=>[...value].some(char=>char.charCodeAt(0)<32||char.charCodeAt(0)===127);
const direction=/[\u202a-\u202e\u2066-\u2069]/u;
export function inspectMessageLink(raw:string,label:string):MessageLink|null{
 if(!raw||raw.length>4096||control(raw)||/\s/u.test(raw)||direction.test(raw))return null;
 let url:URL;try{url=new URL(raw);}catch{return null;}
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
/** Only display copies enter here. Source MIME, quotes and exports stay intact.
 * Body navigation is inert until explicit consent for this message. Only
 * validated destinations can then open a new context without an opener/referrer. */
export function reviewMessageLinks(html:string,help:string,enabled=false):ReviewedMessage{
 const doc=new DOMParser().parseFromString(html,'text/html');const links:MessageLink[]=[];const seen=new Set<string>();let truncated=false;
 for(const anchor of doc.querySelectorAll('a,area')){
  const raw=anchor.getAttribute('href');
  if(raw&&!raw.startsWith('#')){
   const link=inspectMessageLink(raw,anchor.textContent?.trim()??'');
   if(link&&!seen.has(link.href)){seen.add(link.href);if(links.length<100)links.push(link);else truncated=true;}
   if(enabled&&link){anchor.setAttribute('href',link.href);anchor.removeAttribute('aria-disabled');anchor.removeAttribute('title');anchor.setAttribute('target','_blank');anchor.setAttribute('rel','noopener noreferrer');anchor.setAttribute('referrerpolicy','no-referrer');}
   else{anchor.removeAttribute('href');anchor.setAttribute('aria-disabled','true');anchor.setAttribute('title',help);}
  }
  if(!enabled||!anchor.hasAttribute('href')||raw?.startsWith('#'))anchor.removeAttribute('target');anchor.removeAttribute('ping');anchor.removeAttribute('download');
 }
 return {html:'<!doctype html>'+doc.documentElement.outerHTML,links,truncated};
}
