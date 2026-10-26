import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface InboundPolicy {enabled:boolean;cidrs:string[];hostnames:string[]}
export interface InboundDns {servers:string[];timeoutMs:number;cacheSeconds:number;maxHosts:number;maxAddresses:number;concurrency:number}
export interface InboundConfig {admin:InboundPolicy;webmail:InboundPolicy;dns:InboundDns}
export interface InboundHost {hostname:string;addresses:string[]|null;validForSeconds:number|null;error:string|null;usable:boolean}
export interface InboundView {config:InboundConfig;status:{hosts:InboundHost[];refreshing:boolean}}
export const inboundAllowlistApi={read:()=>adminRequest<InboundView>('/security/inbound-allowlist'),save:(config:InboundConfig,etag:string)=>adminRequest<InboundView>('/security/inbound-allowlist',{method:'PUT',body:config,ifMatch:etag})};
export type InboundAllowlistApi=typeof inboundAllowlistApi;
