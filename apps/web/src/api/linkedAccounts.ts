import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {errorDetails,ApiError,type LoginChallenge} from './client.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import type {LinkedAccountContext} from './linkedAccountContext.ts';
import {activeLinkedMailbox,primaryMailboxAuthority} from './linkedMailboxSession.ts';
import {rememberLinkedList,type MailboxLayout} from './mailboxLayout.ts';

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
    &&typeof layout.nicknames==='object'&&layout.nicknames!==null&&!Array.isArray(layout.nicknames)&&Object.values(layout.nicknames).every(name=>typeof name==='string'&&name.length<=256);
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
  async function request<T>(path:string,method='GET',body?:unknown,headers?:Record<string,string>):Promise<T> {
    if(!owner?.storageAuthority||!authority||!preferenceAccountIsCurrent(owner)||controller.signal.aborted)throw new DOMException('Account changed','AbortError');
    const response=await apiFetch(withBase(`/api/accounts${path}`),{
      method,credentials:'same-origin',signal:controller.signal,
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
      rememberLinkedList(value);
      return value;
    },
    link:(input:LinkMailboxInput)=>request<LinkMailboxResult>('/link','POST',input),
    confirm:(challengeId:string,proof:LinkMailboxProof)=>request<{account:LinkedMailbox}>(`/challenges/${encodeURIComponent(challengeId)}/confirm`,'POST',proof),
    /** t48: renew a link's access in place (no unlink); may answer with a second-factor challenge. */
    renew:(id:string,input:RenewMailboxInput)=>request<LinkMailboxResult>(`/${encodeURIComponent(id)}/renew`,'POST',input),
    confirmRenew:(id:string,challengeId:string,proof:LinkMailboxProof)=>request<{account:LinkedMailbox}>(`/${encodeURIComponent(id)}/renew/challenges/${encodeURIComponent(challengeId)}/confirm`,'POST',proof),
    authorize:(id:string)=>request<{context:LinkedAccountContext}>(`/${encodeURIComponent(id)}/switch`,'POST'),
    /** Which mailbox opens first. A preference only: opening it still switches through `authorize`. */
    setPrimary:(linkId:string|null)=>request<{primaryMailbox:string|null}>('/primary','PUT',{linkId}),
    /** Order and nicknames; 409 when another device saved first (re-read, then retry). */
    setLayout:async(layout:MailboxLayout)=>{const saved=await request<{layout:MailboxLayout}>('/layout','PUT',layout);if(!validLayout(saved?.layout))throw new ApiError(502,'Linked mailbox response was invalid');return saved.layout;},
    unlink:(account:LinkedMailbox)=>request<void>(`/${encodeURIComponent(account.id)}`,'DELETE',undefined,{'if-match':JSON.stringify(account.revision)}),
    dispose:()=>{stop();controller.abort();},
  };
}
