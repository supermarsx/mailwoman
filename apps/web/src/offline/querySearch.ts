import type {Email,Mailbox} from '../api/jmap-types.ts';
import {tokenizeSearch,canonicalValue} from '../search/searchQuery.ts';

/** Boolean and field search over content actually present on this device.
 * Absent bodies/filenames cannot match; no result implies a server-wide claim. */
export function matchesCachedSearch(email:Email,input:string,mailboxes:readonly Mailbox[]=[],username?:string):boolean{
 if(input.length>16384)return false;
 const tokens=tokenizeSearch(input);if(tokens.length>1024)return false;let position=0;
 const row=email as unknown as Record<string,unknown>;
 const contains=(value:unknown,text:string)=>String(typeof value==='string'?value:JSON.stringify(value??'')).toLocaleLowerCase().includes(text.toLocaleLowerCase());
 const values=row['bodyValues'] as Record<string,{value?:string}>|undefined;
 const body=Object.values(values??{}).map(value=>(value.value??'').replace(/<[^>]*>/g,' ')).join(' ');
 const text=[email.subject,email.preview,JSON.stringify(email.from),JSON.stringify(email.to),JSON.stringify(row['cc']),body].join(' ');
 function atom(token:Extract<ReturnType<typeof tokenizeSearch>[number],{t:'atom'}>):boolean{
  const value=token.field?canonicalValue(token.field,token.value):token.value;
  if(!token.field||token.field==='text')return contains(text,value);
  if(['from','to','cc','bcc','subject'].includes(token.field))return contains(row[token.field],value);
  if(token.field==='body')return contains(body||email.preview,value);
  if(token.field==='from-domain')return !!email.from?.some(address=>address.email.split('@').at(-1)?.toLocaleLowerCase()===value.toLocaleLowerCase());
  if(token.field==='filename')return contains((row['attachments'] as {name?:string}[]|undefined)?.map(file=>file.name),value);
  if(token.field==='in'){const folder=mailboxes.find(box=>box.id===value||box.role===value||box.name.toLocaleLowerCase()===value.toLocaleLowerCase());return email.mailboxIds[folder?.id??value]===true;}
  if(token.field==='tag')return Object.keys(email.keywords??{}).some(keyword=>keyword.toLocaleLowerCase()===value.toLocaleLowerCase()&&email.keywords?.[keyword]);
  if(token.field==='has')return value==='attachment'?email.hasAttachment===true:value==='calendar'?!!email.mailwomanCalendar:false;
  if(token.field==='pinned')return email.pinned===(value!=='false'&&value!=='no'&&value!=='0');
  if(token.field==='mentions')return !!email.mailwomanMentions?.some(address=>address.toLocaleLowerCase()===(value==='me'?username??'':value).toLocaleLowerCase());
  if(token.field==='before'||token.field==='after'){const date=Date.parse(value),received=Date.parse(email.receivedAt);return Number.isFinite(date)&&(token.field==='before'?received<date:received>=date);}
  if(token.field==='larger'||token.field==='smaller'){const match=/^(\d+)\s*([kmg])?$/i.exec(value);if(!match)return false;const size=Number(match[1])*1024**(({k:1,m:2,g:3} as Record<string,number>)[match[2]?.toLowerCase()??'']??0);return token.field==='larger'?(email.size??0)>size:(email.size??0)<size;}
  if(token.field==='include')return value==='trash';
  if(token.field==='is'){
   if(value==='pinned')return email.pinned===true;
   const keyword:Record<string,string>={read:'$seen',unread:'$seen',flagged:'$flagged',unflagged:'$flagged',draft:'$draft',answered:'$answered',unanswered:'$answered',forwarded:'$forwarded',unforwarded:'$forwarded'};
   if(keyword[value]){const held=email.keywords?.[keyword[value]!]===true;return value.startsWith('un')?!held:held;}
   if(value==='finished')return email.keywords?.['mw-followup-completed']===true;
   if(value==='followup')return email.keywords?.['mw-followup-completed']!==true&&(!!email.followUpAt||email.keywords?.['$flagged']===true);
   if(value==='high-priority'||value==='low-priority')return email.mailwomanPriority===(value==='high-priority'?'high':'low');
  }
  return false;
 }
 function unary(depth:number):boolean{
  if(depth>64)throw new Error('Nested search');
  const token=tokens[position++];if(!token)throw new Error('Incomplete search');
  if(token.t==='not')return !unary(depth+1);
  if(token.t==='lparen'){const value=or(depth+1);if(tokens[position++]?.t!=='rparen')throw new Error('Incomplete group');return value;}
  if(token.t==='atom')return atom(token);
  throw new Error('Unexpected search operator');
 }
 function and(depth:number):boolean{let value=unary(depth);while(position<tokens.length&&!['or','rparen'].includes(tokens[position]!.t)){if(tokens[position]?.t==='and')position++;const right=unary(depth);value=value&&right;}return value;}
 function or(depth:number):boolean{let value=and(depth);while(tokens[position]?.t==='or'){position++;const right=and(depth);value=value||right;}return value;}
 try{if(!tokens.length)return true;const value=or(0);return position===tokens.length&&value;}catch{return false;}
}
