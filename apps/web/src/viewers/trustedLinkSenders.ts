import {isMailbox} from '../api/jmap.ts';

type SenderSettings=Readonly<Record<string,{value:unknown}|undefined>>;
/** Trust exactly one mailbox. Preserve the case-sensitive local part, including
 * plus tags; only the DNS domain is case-normalized. Never accept a domain rule. */
export function normalizeTrustedLinkSender(sender:string):string|null {
 if(/[\p{Cc}\p{Cs}\u061c\u200b-\u200f\u2028-\u202e\u2060-\u206f\ufeff]/u.test(sender)||sender.includes('*')||sender.includes('?'))return null;
 const value=sender.trim();if(!isMailbox(value))return null;
 const at=value.lastIndexOf('@');
 const domain=value.slice(at+1),normalized=value.slice(0,at+1)+(domain.startsWith('[IPv6:')?'[IPv6:'+domain.slice(6).toLowerCase():domain.toLowerCase());
 return isMailbox(normalized)?normalized:null;
}
/** Unknown/malformed settings fail closed. The caller additionally requires an
 * authenticated, ready settings scope and the trusted-links capability. */
export function trustedLinkSenders(values:SenderSettings):string[] {
 const raw=values['links.trustedSenders']?.value;
 if(!Array.isArray(raw)||raw.length>100||raw.some(value=>typeof value!=='string'||normalizeTrustedLinkSender(value)!==value))return [];
 return [...new Set(raw as string[])];
}
export function isTrustedLinkSender(sender:string,values:SenderSettings):boolean {
 const normalized=normalizeTrustedLinkSender(sender);
 return normalized!==null&&trustedLinkSenders(values).includes(normalized);
}
