import {apiFetch} from './http.ts';
import {keyedAttempt,keyedWrite,newIdempotencyKey,type RecoveryOptions} from './recovery.ts';
import {withBase} from './basePath.ts';
import {errorDetails,ApiError,type LoginChallenge} from './client.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import type {LinkedAccountContext} from './linkedAccountContext.ts';
import {activeLinkedMailbox,primaryMailboxAuthority} from './linkedMailboxSession.ts';
import {rememberLinkedList,type MailboxLayout} from './mailboxLayout.ts';
import {validAvatar,type MailboxAvatarSpec} from './mailboxAvatar.ts';

export interface LinkedMailbox {
  id:string;accountId:string;address?:string;displayName:string;
  storageAuthority?:string;status:'ready'|'reauthRequired'|'disabled';
  revision:string;expiresAt?:number;createdAt?:number;
  /** t48: 'automatic' = access renews by itself from the server-sealed credential;
   *  'signIn' = the owner renews it ("Renew access") when it ends. */
  renewal?:'automatic'|'signIn';
  /** Why it does not renew by itself: a code (credentialRejected, factorRequired, zeroAccess, policyOff, ...). */
  renewalReason?:string|null;
}
export interface LinkedMailboxList {
  primary:{accountId:string;address:string;storageAuthority:string};
  accounts:LinkedMailbox[];
  /** The ready link that opens first; null/absent = the sign-in account (t40). */
  primaryMailbox?:string|null;
  /** The stored first mailbox is no longer one of the ready links. */
  primaryFallback?:boolean;
  /** Order and display-only nicknames, per owner (t40 f/g). */
  layout?:MailboxLayout;
  policy:{enabled:boolean;maxAccounts:number;supportedAuthentication:string[];supportedBackends?:string[];reason?:string|null;childRealtime?:boolean;
    /** t48: new links keep a sealed credential; false with a reason code when they will not. */
    storeCredentials?:boolean;storeCredentialsReason?:string|null};
}
export interface LinkMailboxInput {username:string;password:string;displayName?:string;altcha?:string}
/** t48: sign a linked mailbox in again. `username` only when the link does not show an address. */
export interface RenewMailboxInput {username?:string;password:string;altcha?:string}
export type LinkedMailboxChallenge=Omit<LoginChallenge,'pendingToken'|'enrollmentRequired'>&{twofaRequired:true;challengeId:string};
export type LinkMailboxResult={account:LinkedMailbox}|LinkedMailboxChallenge;
export type LinkMailboxProof={method:'totp'|'recovery';code:string}|{method:'webauthn';credentialId:string;clientDataJson:string;authenticatorData:string;signature:string};

function validLayout(value:unknown):value is MailboxLayout {
  const layout=value as MailboxLayout|null;
  return !!layout&&typeof layout.revision==='string'&&Array.isArray(layout.order)&&layout.order.length<=32&&layout.order.every(key=>typeof key==='string')
    &&typeof layout.nicknames==='object'&&layout.nicknames!==null&&!Array.isArray(layout.nicknames)&&Object.values(layout.nicknames).every(name=>typeof name==='string'&&name.length<=256)
    &&(layout.avatars===undefined||(typeof layout.avatars==='object'&&layout.avatars!==null&&!Array.isArray(layout.avatars)&&Object.keys(layout.avatars).length<=32));
}
/** t86: keep only the avatars that pass the same rules the server applies (an older server sends none). */
function cleanLayout(layout:MailboxLayout):MailboxLayout {
  const avatars:Record<string,MailboxAvatarSpec>={};
  for(const [key,raw] of Object.entries(layout.avatars??{})){const spec=validAvatar(raw);if(spec)avatars[key]=spec;}
  return {...layout,avatars};
}
/** t91: a SHA-256 hex digest of `text`, or null where Web Crypto is missing. */
async function requestDigest(text:string):Promise<string|null> {
  const subtle=globalThis.crypto?.subtle;
  if(!subtle)return null;
  try{const hash=await subtle.digest('SHA-256',new TextEncoder().encode(text));return Array.from(new Uint8Array(hash),b=>b.toString(16).padStart(2,'0')).join('');}catch{return null;}
}
/** Management always uses the primary session, never a child-context header.
 * Disposal or any UI authority change aborts pending CSRF/step-up retries too. */
export function createLinkedAccountsApi() {
  const owner=preferenceAccount();
  const authority=primaryMailboxAuthority()??(!activeLinkedMailbox()?owner?.storageAuthority:undefined);
  const controller=new AbortController();
  const stop=observePreferenceAccount(account=>{
    if(account!==owner)controller.abort();
  });
  /** t91: how each write recovers. `keyed`: a create or a credential submission, sent with an
   *  Idempotency-Key (the server replays its first success) and retried on a transient failure
   *  only, never on a refusal such as a 401 for a wrong password. */
  async function request<T>(path:string,method='GET',body?:unknown,headers?:Record<string,string>,mode:'once'|'idempotent'|'keyed'=method==='PUT'?'idempotent':'once'):Promise<T> {
    if(mode!=='keyed')return send<T>(path,method,body,headers,mode==='idempotent'?{recovery:{idempotent:true}}:{});
    // The server classifies a mail-backend timeout after 25s. Let that answer
    // arrive; retry only an unclassified transport failure, with the same key.
    const recovery:Omit<RecoveryOptions,'idempotent'|'idempotencyKey'>=path.endsWith('/renew')?{
      timeoutMs:30_000,attempts:2,offlineWaitMs:30_000,
      nonRetryableCodes:['linkedServerUnreachable','linkedPasswordRejected','linkedRenewLimit'],
    }:{};
    // The key is remembered by a digest of the request (never by the password itself), so pressing
    // the button again after a failure in transit reuses it.
    const identity=await requestDigest(`${method} ${path} ${JSON.stringify(body??null)}`);
    if(identity===null)return send<T>(path,method,body,headers,keyedWrite(newIdempotencyKey(),recovery));
    return keyedAttempt(`linked:${identity}`,key=>send<T>(path,method,body,headers,keyedWrite(key,recovery)));
  }
  async function send<T>(path:string,method:string,body:unknown,headers:Record<string,string>|undefined,recovery:{recovery?:RecoveryOptions}):Promise<T> {
    if(!owner?.storageAuthority||!authority||!preferenceAccountIsCurrent(owner)||controller.signal.aborted)throw new DOMException('Account changed','AbortError');
    const response=await apiFetch(withBase(`/api/accounts${path}`),{
      method,credentials:'same-origin',signal:controller.signal,
      // t82: the PUTs (which mailbox opens first, order and nicknames) set a whole
      // value: retried on a transient failure. t91: switch and unlink too; link,
      // renew and their confirmations with an idempotency key (see `request`).
      ...recovery,
      headers:{'content-type':'application/json','x-mailwoman-storage-authority':authority,...headers},
      ...(body===undefined?{}:{body:JSON.stringify(body)}),
    });
    if(!preferenceAccountIsCurrent(owner)||controller.signal.aborted)throw new DOMException('Account changed','AbortError');
    if(!response.ok)throw new ApiError(response.status,'Linked mailbox request failed',await errorDetails(response));
    if(response.status===204)return undefined as T;
    return await response.json() as T;
  }
  return {
    list:async()=>{
      const value=await request<LinkedMailboxList>('');
      if(!value?.primary||typeof value.primary.address!=='string'||typeof value.primary.accountId!=='string'||value.primary.storageAuthority!==authority
        ||!Array.isArray(value.accounts)||value.accounts.length>16||typeof value.policy?.enabled!=='boolean'||!Number.isInteger(value.policy.maxAccounts)
        ||value.accounts.some(account=>typeof account.id!=='string'||typeof account.revision!=='string'||typeof account.displayName!=='string'||!['ready','reauthRequired','disabled'].includes(account.status)
          ||(account.status==='ready'&&(typeof account.address!=='string'||typeof account.storageAuthority!=='string')))
        ||(value.primaryMailbox!=null&&!value.accounts.some(account=>account.id===value.primaryMailbox&&account.status==='ready'))
        ||(value.primaryFallback!==undefined&&typeof value.primaryFallback!=='boolean')
        ||(value.layout!==undefined&&!validLayout(value.layout)))throw new ApiError(502,'Linked mailbox response was invalid');
      if(value.layout)value.layout=cleanLayout(value.layout);
      rememberLinkedList(value);
      return value;
    },
    link:(input:LinkMailboxInput)=>request<LinkMailboxResult>('/link','POST',input,undefined,'keyed'),
    confirm:(challengeId:string,proof:LinkMailboxProof)=>request<{account:LinkedMailbox}>(`/challenges/${encodeURIComponent(challengeId)}/confirm`,'POST',proof,undefined,'keyed'),
    /** t48: renew a link's access in place (no unlink); may answer with a second-factor challenge. */
    renew:(id:string,input:RenewMailboxInput)=>request<LinkMailboxResult>(`/${encodeURIComponent(id)}/renew`,'POST',input,undefined,'keyed'),
    confirmRenew:(id:string,challengeId:string,proof:LinkMailboxProof)=>request<{account:LinkedMailbox}>(`/${encodeURIComponent(id)}/renew/challenges/${encodeURIComponent(challengeId)}/confirm`,'POST',proof,undefined,'keyed'),
    /** Mints a context for the link; sending it twice only mints another: retried (t91). */
    authorize:(id:string)=>request<{context:LinkedAccountContext}>(`/${encodeURIComponent(id)}/switch`,'POST',undefined,undefined,'idempotent'),
    /** Which mailbox opens first. A preference only: opening it still switches through `authorize`. */
    setPrimary:(linkId:string|null)=>request<{primaryMailbox:string|null}>('/primary','PUT',{linkId}),
    /** Order, nicknames and avatars (t86); 409 when another device saved first (re-read, then retry). */
    setLayout:async(layout:MailboxLayout)=>{const saved=await request<{layout:MailboxLayout}>('/layout','PUT',layout);if(!validLayout(saved?.layout))throw new ApiError(502,'Linked mailbox response was invalid');return cleanLayout(saved.layout);},
    /** Retried (t91). When an earlier attempt removed the link and its answer was lost, the
     *  retry is refused (412 revisionMismatch: the row is gone; or 404): the link no longer
     *  being listed is what was asked, so that counts as done. */
    unlink:async(account:LinkedMailbox)=>{
      try{await request<void>(`/${encodeURIComponent(account.id)}`,'DELETE',undefined,{'if-match':JSON.stringify(account.revision)},'idempotent');}
      catch(error){
        if(error instanceof ApiError&&error.status===404)return;
        if(error instanceof ApiError&&error.status===412){
          const current=await request<LinkedMailboxList>('');
          if(Array.isArray(current?.accounts)&&!current.accounts.some(item=>item.id===account.id))return;
        }
        throw error;
      }
    },
    dispose:()=>{stop();controller.abort();},
  };
}
