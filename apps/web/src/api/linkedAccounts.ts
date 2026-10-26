import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {errorDetails,ApiError,type LoginChallenge} from './client.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import type {LinkedAccountContext} from './linkedAccountContext.ts';
import {activeLinkedMailbox,primaryMailboxAuthority} from './linkedMailboxSession.ts';

export interface LinkedMailbox {
  id:string;accountId:string;address?:string;displayName:string;
  storageAuthority?:string;status:'ready'|'reauthRequired'|'disabled';
  revision:string;expiresAt?:number;createdAt?:number;
}
export interface LinkedMailboxList {
  primary:{accountId:string;address:string;storageAuthority:string};
  accounts:LinkedMailbox[];
  policy:{enabled:boolean;maxAccounts:number;supportedAuthentication:string[];supportedBackends?:string[];reason?:string|null;childRealtime?:boolean};
}
export interface LinkMailboxInput {username:string;password:string;displayName?:string;altcha?:string}
export type LinkedMailboxChallenge=Omit<LoginChallenge,'pendingToken'|'enrollmentRequired'>&{twofaRequired:true;challengeId:string};
export type LinkMailboxResult={account:LinkedMailbox}|LinkedMailboxChallenge;
export type LinkMailboxProof={method:'totp'|'recovery';code:string}|{method:'webauthn';credentialId:string;clientDataJson:string;authenticatorData:string;signature:string};

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
          ||(account.status==='ready'&&(typeof account.address!=='string'||typeof account.storageAuthority!=='string'))))throw new ApiError(502,'Linked mailbox response was invalid');
      return value;
    },
    link:(input:LinkMailboxInput)=>request<LinkMailboxResult>('/link','POST',input),
    confirm:(challengeId:string,proof:LinkMailboxProof)=>request<{account:LinkedMailbox}>(`/challenges/${encodeURIComponent(challengeId)}/confirm`,'POST',proof),
    authorize:(id:string)=>request<{context:LinkedAccountContext}>(`/${encodeURIComponent(id)}/switch`,'POST'),
    unlink:(account:LinkedMailbox)=>request<void>(`/${encodeURIComponent(account.id)}`,'DELETE',undefined,{'if-match':JSON.stringify(account.revision)}),
    dispose:()=>{stop();controller.abort();},
  };
}
