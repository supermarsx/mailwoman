import type {Email} from '../api/jmap-types.ts';
export function relatedSearchValue(message:Email,field:'from'|'subject'):string|null {
 const value=field==='from'?message.from?.[0]?.email:message.subject;
 if(typeof value!=='string'||!value.trim()||value.length>4096||[...value].some(character=>character.charCodeAt(0)<32||character.charCodeAt(0)===127))return null;
 return value.trim();
}

/** Take the mailbox's domain, never a display name or a query fragment. */
export function relatedSenderDomain(message:Email):string|null {
 const from=relatedSearchValue(message,'from');
 if(!from)return null;
 const at=from.lastIndexOf('@');if(at<=0)return null;
 const domain=from.slice(at+1).toLowerCase();
 // DNS/IDNA A-labels only; punctuation cannot become search syntax.
 if(domain.length>253||!domain.split('.').every(label=>/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/u.test(label)))return null;
 return domain;
}
