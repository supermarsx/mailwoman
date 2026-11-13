import {mailHeaderValues} from './mailHeaders.ts';
import {withBase} from './basePath.ts';
import {apiFetch} from './http.ts';

export const listActions=['help','subscribe','unsubscribe','post','owner','archive'] as const;
export type ListAction=typeof listActions[number];
export interface ListLink {href:string;kind:'web'|'mail';host:string}
export interface MailingListCommands {actions:Partial<Record<ListAction,ListLink[]>>;postingForbidden:boolean}
function control(value:string,allowTab=false):boolean {return [...value].some(char=>{const code=char.charCodeAt(0);return code===127||code<32&&!(allowTab&&code===9);});}

/** Header data is untrusted. Parsing never fetches a URL or performs an action. */
export function mailingListCommands(raw:string):MailingListCommands {
 const headers=mailHeaderValues(raw);const actions:MailingListCommands['actions']={};let postingForbidden=false;
 const oversized=new Set<string>();
 for(const line of raw.slice(0,256*1024).split(/\r?\n\r?\n/,1)[0]!.replace(/\r?\n[ \t]+/g,' ').split(/\r?\n/)){
  const match=/^(List-[A-Za-z]+):[ \t]*(.*)$/i.exec(line);if(match&&match[2]!.length>8192)oversized.add(match[1]!.toLowerCase());
 }
 for(const action of listActions){
  const values=headers.get(`list-${action}`);
  // Repeated command fields are ambiguous; do not pick whichever was injected first.
  if(values?.length!==1||oversized.has(`list-${action}`))continue;
  const value=values[0]!;
  if(value.length>8192||control(value,true))continue;
  if(action==='post'&&/^\s*NO\s*$/i.test(value)){postingForbidden=true;continue;}
  const links:ListLink[]=[];let cursor=0;let invalid=false;let needsComma=false;let count=0;
  // RFC2369 alternatives are ordered, enclosed in angle brackets, and comma separated.
  // Comments outside URLs are ignored, with nesting bounded to avoid hostile input.
  while(cursor<value.length){
   while(/\s/.test(value[cursor]??'')&&cursor<value.length)cursor++;
   if(value[cursor]==='('){let depth=1;cursor++;while(cursor<value.length&&depth){const c=value[cursor++];if(c==='\\'){cursor++;continue;}if(c==='(')depth++;if(c===')')depth--;if(depth>8){invalid=true;break;}}if(depth||invalid){invalid=true;break;}continue;}
   if(cursor===value.length)break;
   if(needsComma){if(value[cursor]!==','){invalid=true;break;}cursor++;needsComma=false;continue;}
   if(value[cursor]!=='<'){invalid=true;break;}
   const end=value.indexOf('>',cursor+1);if(end<0){invalid=true;break;}
   const link=safeListLink(value.slice(cursor+1,end));cursor=end+1;needsComma=true;count++;
   if(link&&!links.some(old=>old.href===link.href))links.push(link);
   if(count>16){invalid=true;break;}
  }
  if(!invalid&&needsComma&&links.length)actions[action]=links;
 }
 return {actions,postingForbidden};
}

function safeListLink(raw:string):ListLink|null {
 // Never repair embedded whitespace, accept credentials, or open executable schemes.
 if(raw.length>2048||control(raw)||/[\s<>]/u.test(raw))return null;
 let url:URL;try{url=new URL(raw);}catch{return null;}
 if(url.username||url.password)return null;
 if(url.protocol==='https:'||url.protocol==='http:')return {href:url.href,kind:'web',host:url.host};
 if(url.protocol!=='mailto:')return null;
 let address:string;try{address=decodeURIComponent(url.pathname);}catch{return null;}
 if(control(address)||!/^[^\s@,;<>?]+@[^\s@,;<>?]+\.[^\s@,;<>?]+$/u.test(address))return null;
 // No hidden recipients/header injection through an ostensibly simple command.
 const seen=new Set<string>();
 for(const [key,value] of url.searchParams){const lower=key.toLowerCase();if(seen.has(lower)||!['subject','body'].includes(lower)||lower==='subject'&&control(value)||[...value].some(char=>char.charCodeAt(0)===0))return null;seen.add(lower);}
 if(url.hash)return null;
 return {href:url.href,kind:'mail',host:address};
}

/** Safe, explicit compose input; never invokes a mailto handler or sends mail. */
export function listMailDraft(link:ListLink):{to:string;subject:string;bodyText:string;bodyHtml:string}|null {
 const checked=safeListLink(link.href);if(checked?.kind!=='mail')return null;
 const url=new URL(checked.href);let subject='',bodyText='';
 for(const [key,value] of url.searchParams){if(key.toLowerCase()==='subject')subject=value;if(key.toLowerCase()==='body')bodyText=value;}
 const escaped=bodyText.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
 return {to:decodeURIComponent(url.pathname),subject,bodyText,bodyHtml:bodyText?`<p>${escaped.replace(/\r\n|\r|\n/g,'<br>')}</p>`:''};
}

/** The list's display identity from List-Id (RFC 2919): its phrase when there is
 * one ("Example news" <news.example.org>), else the id itself. Untrusted text:
 * capped, single-valued, no control characters; `null` when absent or unusable. */
export function mailingListName(raw:string):string|null {
 const values=mailHeaderValues(raw).get('list-id');if(values?.length!==1)return null;
 const value=values[0]!.trim();if(!value||value.length>512||control(value,true))return null;
 const match=/^(.*?)<([^<>\s]+)>\s*$/.exec(value);
 const phrase=(match?match[1]!:'').trim().replace(/^"(.*)"$/,'$1').replace(/\\(.)/g,'$1').trim();
 const id=(match?match[2]!:value).trim();
 const name=phrase||id;return name.length>0&&name.length<=200?name:null;
}

/** The List-Id value itself (RFC 2919): the `<id>` part, else the bare value. Untrusted; `null` when absent or unusable. */
export function mailingListId(raw:string):string|null {
 const values=mailHeaderValues(raw).get('list-id');if(values?.length!==1)return null;
 const value=values[0]!.trim();if(!value||value.length>512||control(value,true))return null;
 const match=/^(.*?)<([^<>\s]+)>\s*$/.exec(value);
 const id=(match?match[2]!:value).trim();
 return id.length>0&&id.length<=200&&!/[\s<>]/u.test(id)?id:null;
}

export interface UnsubscribeMethods {
 /** An https List-Unsubscribe URL the server may POST to (RFC 8058): needs a List-Unsubscribe-Post header. */
 oneClick:ListLink|null;
 /** The first web (http/https) alternative, opened only after review. */
 web:ListLink|null;
 /** The first mailto alternative, composed only after review. */
 mail:ListLink|null;
}
/** The unsubscribe methods the message offers. Nothing is fetched or sent. */
export function unsubscribeMethods(raw:string):UnsubscribeMethods {
 const links=mailingListCommands(raw).actions.unsubscribe??[];
 const post=mailHeaderValues(raw).get('list-unsubscribe-post');
 const declared=post?.length===1&&/(^|[\s;,])List-Unsubscribe=One-Click(\s|;|,|$)/i.test(post[0]!);
 const web=links.find(link=>link.kind==='web')??null;
 const secure=links.find(link=>link.kind==='web'&&link.href.startsWith('https:'))??null;
 return {oneClick:declared?secure:null,web,mail:links.find(link=>link.kind==='mail')??null};
}

/** Whether the message carries list headers worth a badge (List-Id or List-Unsubscribe). */
export function hasListHeaders(raw:string):boolean {
 return mailingListId(raw)!==null||unsubscribeMethods(raw).web!==null||unsubscribeMethods(raw).mail!==null;
}

/** Keep only links that are safe to show as text: http(s) and mailto. */
export function safeListHref(href:string):ListLink|null {return safeListLink(href);}

export type OneClickResult={ok:true}|{ok:false;reason:string};
/** RFC 8058 through the server (`POST /api/mail/list-unsubscribe`): the list host sees the server, not the reader. */
export async function unsubscribeOneClick(url:string):Promise<OneClickResult> {
 try{
  const response=await apiFetch(withBase('/api/mail/list-unsubscribe'),{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({url})});
  const body=await response.json().catch(()=>({})) as {ok?:unknown;reason?:unknown};
  if(response.ok&&body.ok===true)return {ok:true};
  return {ok:false,reason:typeof body.reason==='string'?body.reason:'rejected'};
 }catch(error){return {ok:false,reason:error instanceof Error&&error.name==='LinkedContextUnsupportedError'?'refused':error instanceof Error&&error.name==='AbortError'?'cancelled':'network'};}
}
