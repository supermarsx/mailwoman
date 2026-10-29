import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface SshPin {algorithm:string;sha256:string}
export interface SshTarget {host:string;port:number;allowPrivate:boolean}
export interface SshTunnelConfig {id:string;name:string;host:string;port:number;username:string;enabled:boolean;allowPrivate:boolean;hostKeyPins:SshPin[];targets:SshTarget[];keepaliveSeconds:number;connectTimeoutMs:number;maxChannels:number}
export interface SshTunnel extends SshTunnelConfig {revision:string;etag:string;credentialConfigured:boolean;authKind:'privateKey'|'password'|null;publicKey:string|null;createdAt:number;updatedAt:number}
export type SshCredential={action:'keep'}|{action:'clear'}|{action:'generate'}|{action:'set';kind:'privateKey';privateKey:string;passphrase?:string}|{action:'set';kind:'password';password:string};
export interface SshDiagnostic {ok:boolean;phase:string;code?:string;latencyMs:number;latencyUs?:number;pinnedFingerprint?:string}
export type SshInspection = Pick<SshTunnelConfig,'host'|'port'|'allowPrivate'|'connectTimeoutMs'>;
export interface SshTunnelApi {inspectDraft:(authority:SshInspection)=>Promise<SshPin>;list:()=>Promise<SshTunnel[]>;get:(id:string)=>Promise<SshTunnel>;save:(config:SshTunnelConfig,credential:SshCredential,etag?:string,confirmAuthorityChange?:boolean)=>Promise<SshTunnel>;remove:(row:SshTunnel)=>Promise<void>;inspect:(row:SshTunnel)=>Promise<SshPin>;test:(row:SshTunnel)=>Promise<SshDiagnostic>}
const root='/tunnels';
function tagged(row:Omit<SshTunnel,'etag'>,etag?:string|null):SshTunnel{return {...row,etag:etag??`"${row.revision}"`};}
export const sshTunnelApi:SshTunnelApi={
 async inspectDraft(authority){return(await adminRequest<SshPin>(`${root}/inspect-host-key`,{method:'POST',body:authority})).data;},
 async list(){const response=await adminRequest<{tunnels:Omit<SshTunnel,'etag'>[]}>(root);return response.data.tunnels.map(row=>tagged(row));},
 async get(id){const response=await adminRequest<Omit<SshTunnel,'etag'>>(`${root}/${encodeURIComponent(id)}`);return tagged(response.data,response.etag);},
 async save(config,credential,etag,confirmAuthorityChange){const {id,...editable}=config;const response=await adminRequest<Omit<SshTunnel,'etag'>>(etag?`${root}/${encodeURIComponent(id)}`:root,{method:etag?'PUT':'POST',body:{...editable,credential,...(confirmAuthorityChange?{confirmAuthorityChange:true}:{})},...(etag?{ifMatch:etag}:{})});return tagged(response.data,response.etag);},
 async remove(row){await adminRequest(`${root}/${encodeURIComponent(row.id)}`,{method:'DELETE',ifMatch:row.etag});},
 async inspect(row){return(await adminRequest<SshPin>(`${root}/${encodeURIComponent(row.id)}/fetch-host-key`,{method:'POST',ifMatch:row.etag})).data;},
 async test(row){return(await adminRequest<SshDiagnostic>(`${root}/${encodeURIComponent(row.id)}/test`,{method:'POST',ifMatch:row.etag})).data;},
};
