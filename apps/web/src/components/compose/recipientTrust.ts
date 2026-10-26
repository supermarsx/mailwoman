import { parseRecipients } from '../../api/jmap.ts';
/** Exact DNS/IDNA domain matching, never suffix or substring matching. */
export function normalizeTrustedDomain(value: string): string | null {
  const domain=value.trim().replace(/\.$/,'');
  if(!domain || /[\s/@:\\?#*%]/u.test(domain))return null;
  try{const hostname=new URL(`https://${domain}`).hostname.toLowerCase();return hostname.length<=253&&hostname.split('.').every(label=>label.length<=63&&/^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/.test(label))?hostname:null;}catch{return null;}
}
export function externalRecipients(fields: readonly string[], sender: string, trusted: readonly string[]): string[] {
  const own=normalizeTrustedDomain(sender.slice(sender.lastIndexOf('@')+1));
  const domains=new Set(trusted.map(normalizeTrustedDomain).filter((value):value is string=>value!==null));
  if(sender.includes('@')&&own)domains.add(own);
  return [...new Set(fields.flatMap(parseRecipients).map(address=>address.email).filter(address=>{
    const at=address.lastIndexOf('@');const domain=at>=0?normalizeTrustedDomain(address.slice(at+1)):null;
    return domain===null||!domains.has(domain);
  }))];
}
