import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {preferenceAccountIsCurrent,type PreferenceAccount} from './accountPreferenceLifecycle.ts';
import {isMailbox} from './jmap.ts';
import {t} from '../i18n/index.ts';

/** Resolve each explicit send against current server authority, never a cached identity. */
export async function resolveSender(email:string,owner:PreferenceAccount|null):Promise<{email:string;identityId:string|null}>{
 const value=email.trim();
 if(!isMailbox(value))throw new Error(t('mail-compose-from-invalid'));
 if(!owner?.storageAuthority||!preferenceAccountIsCurrent(owner))throw new Error(t('mail-compose-account-changed'));
 let response:Response;
 try {response=await apiFetch(withBase('/api/account/sender/resolve'),{method:'POST',headers:{'content-type':'application/json','x-mailwoman-account':owner.id,'x-mailwoman-storage-authority':owner.storageAuthority},body:JSON.stringify({email:value})});}
 catch {throw new Error(t('mail-compose-from-unavailable'));}
 if(!preferenceAccountIsCurrent(owner))throw new Error(t('mail-compose-account-changed'));
 if(!response.ok)throw new Error(t(response.status===403?'mail-compose-from-refused':'mail-compose-from-unavailable'));
 const result:unknown=await response.json();
 if(!result||typeof result!=='object')throw new Error(t('mail-compose-from-unavailable'));
 const resolved=result as Record<string,unknown>;
 if(typeof resolved.email!=='string'||resolved.email!==value||(resolved.identityId!==null&&(typeof resolved.identityId!=='string'||!resolved.identityId)))throw new Error(t('mail-compose-from-unavailable'));
 return {email:resolved.email,identityId:resolved.identityId as string|null};
}
