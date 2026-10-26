import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
export type CloudPimKind='calendar'|'tasks'|'contacts';
export interface CloudCollection {id:string;instanceId:string;kind:CloudPimKind;enabled:boolean;name:string;ready:boolean}
export interface CloudDiscovery {href:string;name:string;kind:CloudPimKind}
export interface CloudPimView {accountId:string;supported:boolean;collections:CloudCollection[]}
async function call<T>(path:string,method:string,body?:unknown):Promise<T>{
  const response=await apiFetch(withBase(`/api/account/nextcloud/pim${path}`),{method,credentials:'same-origin',headers:body===undefined?{}:{'content-type':'application/json'},body:body===undefined?null:JSON.stringify(body)});
  if(!response.ok)throw new Error(`Cloud PIM request failed (${response.status})`);
  return (response.status===204?undefined:await response.json()) as T;
}
export const nextcloudPimApi={
  list:(accountId:string)=>call<CloudPimView>(`?accountId=${encodeURIComponent(accountId)}`,'GET'),
  discover:(accountId:string,instance:string,kind:CloudPimKind)=>call<{accountId:string;collections:CloudDiscovery[]}>(`/discover/${encodeURIComponent(instance)}`,'POST',{accountId,kind}),
  apply:(accountId:string,instance:string,item:CloudDiscovery)=>call<{collectionId:string}>(`/apply/${encodeURIComponent(instance)}`,'POST',{accountId,kind:item.kind,href:item.href}),
  disconnect:(accountId:string,id:string)=>call<void>(`/collections/${encodeURIComponent(id)}`,'DELETE',{accountId}),
  sync:(accountId:string)=>call<{ok:boolean}>('/sync','POST',{accountId}),
};
export type NextcloudPimApi=typeof nextcloudPimApi;
