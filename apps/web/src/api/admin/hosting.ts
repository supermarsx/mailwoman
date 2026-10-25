import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export type HostingProvider = 'virtualmin' | 'hestia';
export interface HostingConfig {
  id:string; name:string; provider:HostingProvider; baseUrl:string; enabled:boolean;
  username:string; owner:string; accessKey:string; allowPrivate:boolean;
  connection:{kind:'direct'}|{kind:'ssh';tunnelId:string};
  tls:{serverName?:string|null;caPem?:string|null;minVersion:'1.2'|'1.3'};
  allowDomainLifecycle:boolean;
}
export type HostingSecret={action:'keep'}|{action:'clear'}|{action:'set';value:string};
export interface HostingConnection extends HostingConfig { credential:{set:boolean};etag:string;updatedAt:number }
export type HostingOperation=
 | {action:'listDomains'}
 | {action:'listMailboxes';domain:string}
 | {action:'createDomain';domain:string;parent?:string;owner?:string;password?:string}
 | {action:'setDomainEnabled';domain:string;enabled:boolean}
 | {action:'deleteDomain';domain:string}
 | {action:'createMailbox';domain:string;account:string;password:string;quotaMiB?:number;displayName?:string}
 | {action:'setMailboxQuota';domain:string;account:string;quotaMiB:number}
 | {action:'resetMailboxPassword';domain:string;account:string;password:string}
 | {action:'setMailboxEnabled';domain:string;account:string;enabled:boolean}
 | {action:'deleteMailbox';domain:string;account:string}
 | {action:'addAlias'|'removeAlias';domain:string;account:string;alias:string};
export interface HostingDomain {name:string;owner?:string|null;enabled?:boolean|null}
export interface HostingMailbox {account:string;address:string;enabled?:boolean|null;quotaMiB?:number|null;aliases:string[]}
export interface HostingResult {domains?:HostingDomain[];mailboxes?:HostingMailbox[];changed:boolean}
export interface HostingApi {
 list:()=>Promise<HostingConnection[]>;
 get:(id:string)=>Promise<HostingConnection>;
 save:(config:HostingConfig,credential:HostingSecret,etag?:string)=>Promise<HostingConnection>;
 remove:(connection:HostingConnection)=>Promise<void>;
 perform:(connection:HostingConnection,operation:HostingOperation,confirmation?:string)=>Promise<HostingResult>;
}
const root='/integrations/hosting/connections';
export const hostingApi:HostingApi={
 async list(){return(await adminRequest<{connections:HostingConnection[]}>(root)).data.connections;},
 async get(id){return(await adminRequest<HostingConnection>(`${root}/${encodeURIComponent(id)}`)).data;},
 async save(config,credential,etag){return(await adminRequest<HostingConnection>(etag?`${root}/${encodeURIComponent(config.id)}`:root,{method:etag?'PUT':'POST',body:{config,credential},...(etag?{ifMatch:etag}:{})})).data;},
 async remove(connection){await adminRequest(`${root}/${encodeURIComponent(connection.id)}`,{method:'DELETE',ifMatch:connection.etag});},
 async perform(connection,operation,confirmation){return(await adminRequest<HostingResult>(`${root}/${encodeURIComponent(connection.id)}/actions`,{method:'POST',body:{operation,...(confirmation?{confirmation}:{})},ifMatch:connection.etag})).data;},
};
