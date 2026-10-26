import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {adminUrl} from './adminPath.ts';
import {AdminProblemError} from '../screens/Admin/shell/adminApi.ts';

export interface CloudInstance {id:string;name:string;baseUrl:string;enabled:boolean;etag:string;caPem?:string}
export interface CloudLink {instanceId:string;username:string;ready:boolean;etag:string}
export interface CloudAccount {accountId:string;allowed?:boolean;instances:CloudInstance[];links:CloudLink[];maxFileBytes:number}
export interface CloudInstances {instances:CloudInstance[];maxInstances:number;maxFileBytes:number}
export interface CloudWrite {name:string;baseUrl:string;enabled:boolean;caPem?:string}
async function request<T>(path:string,method='GET',body?:unknown,etag?:string,create=false):Promise<T>{
  const response=await apiFetch(path,{method,credentials:'same-origin',headers:{...(body===undefined?{}:{'content-type':'application/json'}),...(etag?{'if-match':etag}:{}),...(create?{'if-none-match':'*'}:{})},body:body===undefined?null:JSON.stringify(body)});
  const text=await response.text();const data:unknown=text?JSON.parse(text):null;
  if(!response.ok)throw new AdminProblemError(response.status,typeof data==='object'&&data!==null?data as Record<string,unknown>:{},response.headers.get('etag'));
  return data as T;
}
export interface NextcloudAccountsApi {
  account():Promise<CloudAccount>;
  link(id:string,username:string,appPassword:string,accountId:string,etag?:string):Promise<CloudLink>;
  unlink(id:string,accountId:string,etag:string):Promise<void>;
  instances():Promise<CloudInstances>;
  save(id:string,value:CloudWrite,etag?:string):Promise<CloudInstance>;
}
export const nextcloudAccountsApi:NextcloudAccountsApi={
  account:()=>request(withBase('/api/account/nextcloud')),
  link:(id,username,appPassword,accountId,etag)=>request(withBase(`/api/account/nextcloud/links/${encodeURIComponent(id)}`),'PUT',{username,appPassword,accountId},etag),
  unlink:(id,accountId,etag)=>request(withBase(`/api/account/nextcloud/links/${encodeURIComponent(id)}`),'DELETE',{accountId},etag),
  instances:()=>request(adminUrl('/nextcloud/instances')),
  save:(id,value,etag)=>request(adminUrl(`/nextcloud/instances/${encodeURIComponent(id)}`),'PUT',value,etag,etag===undefined),
};
