import {normalizeTrustedDomain} from './recipientTrust.ts';

/** `mail.replyImages`: whether a reply or forward carries the original's images. */
export type ReplyImagePolicy='ask'|'allow'|'trusted'|'deny';
export type ReplyImageDecision='include'|'ask'|'omit';
export const REPLY_IMAGE_POLICIES:readonly ReplyImagePolicy[]=['ask','allow','trusted','deny'];
export const isReplyImagePolicy=(value:unknown):value is ReplyImagePolicy=>REPLY_IMAGE_POLICIES.includes(value as ReplyImagePolicy);

/** The normalised domain of a sender address, or null. */
export function senderDomain(address:string):string|null{
 const at=address.lastIndexOf('@');return at<0?null:normalizeTrustedDomain(address.slice(at+1));
}
/** Exact match only; subdomains are never implicitly trusted. */
export function domainTrusted(domain:string|null,trusted:readonly string[]):boolean{
 return domain!==null&&trusted.some(value=>normalizeTrustedDomain(value)===domain);
}
/** An unknown or malformed policy value asks, never allows. */
export function replyImageDecision(policy:unknown,domain:string|null,trusted:readonly string[]):ReplyImageDecision{
 switch(isReplyImagePolicy(policy)?policy:'ask'){
  case 'allow':return 'include';
  case 'deny':return 'omit';
  case 'trusted':return domainTrusted(domain,trusted)?'include':'ask';
  default:return 'ask';
 }
}
/** Whether sanitised reader HTML refers to any image at all. */
export function hasQuoteImages(html:string):boolean{
 return /<img\b|mw-rc:\d|cid:|url\(\s*["']?(?:data:image|cid:|mw-rc:)|background\s*=/i.test(html);
}
