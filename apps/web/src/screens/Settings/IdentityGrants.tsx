import { createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { loadCatalog, t } from '../../i18n';
import * as css from '../Consent/consent.css.ts';
type Grants={grants:{clientId:string;name:string;scopes:string[];updatedAt:number}[];csrfToken:string};
export function IdentityGrants():JSX.Element{
 onMount(()=>void loadCatalog('auth'));const[error,setError]=createSignal('');const[busy,setBusy]=createSignal('');
 const[data,{refetch}]=createResource(async():Promise<Grants>=>{const response=await apiFetch(`${basePath()}/oidc/grants`,{credentials:'same-origin',cache:'no-store'});if(!response.ok)throw new Error(t('auth-identity-expired'));return await response.json() as Grants;});
 async function revoke(id:string):Promise<void>{setBusy(id);setError('');try{const response=await apiFetch(`${basePath()}/oidc/grants/${encodeURIComponent(id)}`,{method:'DELETE',credentials:'same-origin',headers:{'x-csrf-token':data()?.csrfToken??''}});if(!response.ok)throw new Error(t('auth-identity-expired'));await refetch();}catch(cause){setError(cause instanceof Error?cause.message:t('auth-identity-expired'));}finally{setBusy('');}}
 return <section aria-label={t('auth-identity-grants')}><h3>{t('auth-identity-grants')}</h3><p>{t('auth-identity-grants-help')}</p><Show when={data.loading}><p role="status">{t('auth-consent-loading')}</p></Show><Show when={data.error||error()}><p role="alert">{error()||t('auth-identity-expired')}</p><button onClick={()=>void refetch()}>{t('auth-identity-retry')}</button></Show><Show when={data()?.grants.length===0}><p>{t('auth-identity-no-grants')}</p></Show><For each={data()?.grants}>{grant=><article class={css.card}><h4>{grant.name}</h4><p>{grant.scopes.join(', ')}</p><button disabled={!!busy()} onClick={()=>void revoke(grant.clientId)}>{t('auth-identity-revoke')}</button></article>}</For></section>;
}
