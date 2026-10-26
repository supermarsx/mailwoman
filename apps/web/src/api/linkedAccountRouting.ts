/** Mirrors the audited server scope. Unknown mailbox APIs fail closed while a
 * child is selected; omitting a selector would execute against the primary. */
export type LinkedRoute='mailbox'|'primary'|'public'|'signed'|'unsupported';
const mailbox=new Set([
  '/api/me','/jmap/session','/jmap/api','/api/settings','/api/account/preferences',
  '/api/account/appearance','/api/account/signatures','/api/account/notifications',
  '/api/account/saved-searches','/api/account/identities','/api/account/sender/resolve',
  '/api/account/mdn-policy','/api/account/drafts','/api/account/drafts/resolve',
  '/api/account/session/activity','/api/attachments/policy','/api/security/step-up',
  '/api/security/step-up/passkey','/api/security/dlp/config','/api/security/watermark',
  '/api/rc/policy','/api/rc/sign','/api/rc/stats','/api/sanitize','/api/policy',
  '/api/mail/delivery-status','/api/attachments/preview',
]);
function segments(path:string,prefix:string,count:number):boolean {
  if(!path.startsWith(prefix))return false;
  const tail=path.slice(prefix.length).split('/');
  return tail.length===count&&tail.every(Boolean);
}
export function linkedUnsignedResource(path:string):boolean {
  return segments(path,'/jmap/download/',3)||segments(path,'/api/export/',1);
}
export function linkedRoute(path:string):LinkedRoute {
  if(mailbox.has(path)||['/api/settings/','/api/account/signatures/','/api/account/saved-searches/','/api/account/identities/','/api/account/drafts/','/jmap/upload/'].some(prefix=>segments(path,prefix,1))
    ||(path.endsWith('/cleanup')&&segments(path,'/api/account/drafts/',2))||linkedUnsignedResource(path))return 'mailbox';
  if(path==='/api/accounts'||path==='/api/accounts/link'||segments(path,'/api/accounts/',1)
    ||(path.endsWith('/switch')&&segments(path,'/api/accounts/',2))
    ||(path.endsWith('/confirm')&&segments(path,'/api/accounts/challenges/',2))
    ||path==='/api/logout')return 'primary';
  if(path==='/api/public-config'||path==='/api/signin/challenge')return 'public';
  if(['/api/rc/i/','/api/rc/p/','/api/rc/f/','/api/rc/d/'].some(prefix=>segments(path,prefix,1)))return 'signed';
  return 'unsupported';
}

/** Only cookie-authenticated unsigned download URLs carry mwLink. Signed URLs
 * and external destinations must never acquire a selector or parent secrets. */
export function linkedResourceUrl(input:string,linkId:string,serverBase:string):string {
  const base=new URL(serverBase);
  const url=new URL(input,base);
  const prefix=base.pathname.replace(/\/$/,'');
  if(url.origin!==base.origin||url.username||url.password||!url.pathname.startsWith(`${prefix}/`))throw new Error('Unsupported linked resource');
  const path=url.pathname.slice(prefix.length);
  if(!linkedUnsignedResource(path)||!linkId||url.searchParams.has('mwLink'))throw new Error('Unsupported linked resource');
  url.searchParams.set('mwLink',linkId);
  return url.href;
}
