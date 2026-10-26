import type {InboundConfig,InboundDns} from '../../api/admin/inboundAllowlist.ts';
export interface InboundDraft {admin:{enabled:boolean;cidrs:string;hostnames:string};webmail:{enabled:boolean;cidrs:string;hostnames:string};dns:Omit<InboundDns,'servers'>&{servers:string}}
export const dnsLimits={timeoutMs:[100,5000],cacheSeconds:[30,3600],maxHosts:[1,64],maxAddresses:[1,32],concurrency:[1,4]} as const;
const lines=(value:string)=>value.split(/\r?\n/u).map(line=>line.trim()).filter(Boolean);
export function inboundDraft(config:InboundConfig):InboundDraft{return {admin:{...config.admin,cidrs:config.admin.cidrs.join('\n'),hostnames:config.admin.hostnames.join('\n')},webmail:{...config.webmail,cidrs:config.webmail.cidrs.join('\n'),hostnames:config.webmail.hostnames.join('\n')},dns:{...config.dns,servers:config.dns.servers.join('\n')}};}
export function inboundConfig(draft:InboundDraft):InboundConfig{return {admin:{...draft.admin,cidrs:lines(draft.admin.cidrs),hostnames:lines(draft.admin.hostnames)},webmail:{...draft.webmail,cidrs:lines(draft.webmail.cidrs),hostnames:lines(draft.webmail.hostnames)},dns:{...draft.dns,servers:lines(draft.dns.servers)}};}
export function inboundProblem(config:InboundConfig):string|null {
 for(const [key,bounds] of Object.entries(dnsLimits)){const n=config.dns[key as keyof typeof dnsLimits];if(!Number.isInteger(n)||n<bounds[0]||n>bounds[1])return 'admin-inbound-invalid-dns';}
 if(config.dns.servers.length>4)return 'admin-inbound-invalid-dns';
 const hosts=new Set<string>();for(const policy of [config.admin,config.webmail]){if(policy.enabled&&!policy.cidrs.length)return 'admin-inbound-rescue-required';if(policy.cidrs.length>128)return 'admin-inbound-too-many';for(const host of policy.hostnames){if(host.length>253||!host.includes('.')||/^[\d.]+$/u.test(host)||!host.split('.').every(label=>/^[a-z\d](?:[a-z\d-]{0,61}[a-z\d])?$/u.test(label)))return 'admin-inbound-invalid-host';hosts.add(host);}}
 return hosts.size>config.dns.maxHosts?'admin-inbound-too-many':null;
}
