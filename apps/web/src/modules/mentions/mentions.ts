import {parse,defaultTreeAdapter as tree,type DefaultTreeAdapterTypes as T} from 'parse5';
import {isMailbox,parseRecipients} from '../../api/jmap.ts';
import {rankSuggestions,suggestionDisplay,type ContactSuggestion} from '../contacts/autocomplete.ts';
import {ownText as authoredText} from '../../components/compose/sendChecks.ts';
import type {ContactCard} from '../../api/pim-types.ts';

export interface MentionQuery {from:number;to:number;query:string}
/** Cursor-local token, never the @ inside an email address or a URL. */
export function mentionQuery(text:string,caret=text.length):MentionQuery|null {
 const before=text.slice(0,caret),match=/(?:^|[\s([{])@([\p{L}\p{N} ._'’+-]{0,64})$/u.exec(before);
 if(!match)return null;const query=match[1]!;
 return {from:caret-query.length-1,to:caret,query};
}
export function mergeMentionSuggestions(...lists:readonly ContactSuggestion[][]):ContactSuggestion[]{const seen=new Set<string>();return lists.flat().filter(result=>{const email=result.email.toLowerCase();if(!isMailbox(result.email)||seen.has(email))return false;seen.add(email);return true;}).slice(0,8);}
export function mentionSuggestions(cards:readonly ContactCard[],query:string):ContactSuggestion[]{
 const results=query.trim()?rankSuggestions(cards,query,30):cards.flatMap(card=>card.emails.filter(e=>isMailbox(e.value)).slice(0,1).map(email=>({cardId:card.id,name:card.name.full,email:email.value,display:suggestionDisplay(card.name.full,email.value),isFavorite:card.isFavorite,score:card.isFavorite?10000:1}))).sort((a,b)=>b.score-a.score||a.display.localeCompare(b.display));
 const seen=new Set<string>();return results.filter(result=>{const email=result.email.toLowerCase();if(!isMailbox(result.email)||seen.has(email))return false;seen.add(email);return true;}).slice(0,8);
}
/** Explicit selection only: keep a Bcc recipient private and never duplicate it in To. */
export function toWithMention(to:string,cc:string,bcc:string,pick:Pick<ContactSuggestion,'name'|'email'>,separator=','):string {
 if(!isMailbox(pick.email)||parseRecipients([to,cc,bcc].join(',')).some(value=>value.email.toLowerCase()===pick.email.toLowerCase()))return to;
 const prefix=to.trim().replace(/[,;]\s*$/,'');return(prefix?prefix+separator+' ':'')+suggestionDisplay(pick.name,pick.email);
}
function mailto(href:string):string|null {try{const url=new URL(href);if(url.protocol!=='mailto:')return null;const email=decodeURIComponent(url.pathname);return isMailbox(email)?email.toLowerCase():null;}catch{return null;}}
export function explicitMentionAddresses(text:string):string[]{
 const out=new Set<string>();for(const match of text.matchAll(/(?:^|[^\p{L}\p{N}._%+-])@([^\s<>,;()[\]{}"']+@[^\s<>,;()[\]{}"']+)/gu)){const address=match[1]!.replace(/[.!?:]+$/,'');if(isMailbox(address))out.add(address.toLowerCase());}return [...out];
}
const ignored=new Set(['blockquote','script','style','head','template']);
/** Pure HTML parsing: resource URLs are never handed to the browser DOM. A
 * portable @-labelled mailto anchor supplies the address even with a display name. */
export function mentionedAddresses(html:string,text=''):string[]{
 const out=new Set<string>();let ownText='';
 const walk=(node:T.Node):void=>{
  if(tree.isTextNode(node)){ownText+=node.value;return;}
  if('tagName'in node){
   if(ignored.has(node.tagName)||node.attrs.some(a=>(a.name==='class'&&/(?:^|\s)(?:gmail_quote|gmail_signature|moz-signature)(?:\s|$)/.test(a.value))||a.name==='data-mw-signature'))return;
   if(node.tagName==='a'){const href=node.attrs.find(a=>a.name==='href')?.value;let label='';const collect=(n:T.Node):void=>{if(tree.isTextNode(n))label+=n.value;else if('childNodes'in n)n.childNodes.forEach(collect);};collect(node);const email=href?mailto(href):null;if(email&&label.trim().startsWith('@'))out.add(email);}
  }
  if('childNodes'in node)node.childNodes.forEach(walk);if('tagName'in node&&['p','div','br','li','tr'].includes(node.tagName))ownText+='\n';
 };
 if(html)walk(parse(html));for(const address of explicitMentionAddresses(authoredText(html?ownText:text)))out.add(address);return [...out];
}
export {mentionsAny} from './metadata.ts';
