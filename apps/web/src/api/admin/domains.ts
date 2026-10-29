import type { PasswordAuthentication, PasswordMechanism } from "./domain-auth.ts";
import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export type UsernameFormat = { format: 'fullEmail' | 'localUser' } | { format: 'domainUser'; domain: string } | { format: 'userRealm'; realm: string } | { format: 'custom'; template: string; realm?: string };
export interface ConnectionSecurity { mode: 'fixed' | 'auto'; allowed: ('implicit' | 'starttls' | 'none')[]; allowPlaintext: boolean; serverName?: string; caPem?: string }
export interface DomainBackend { id: string; role: 'mail' | 'submission' | 'rules'; protocol: 'imap' | 'pop3' | 'jmap' | 'smtp' | 'sieve' | 'ews'; enabled?: boolean; preferred?: boolean; priority?: number; endpoint?: DomainEndpoint; sessionUrl?: string; ews?: { endpointUrl: string; authentication: 'basic'; caPem?: string }; username?: UsernameFormat }
export interface DomainEndpoint { authentication?: PasswordAuthentication; security?: ConnectionSecurity; host: string; port: number; tls: 'implicit' | 'starttls' | 'none'; auth: 'password'; allowPrivate?: boolean }
export interface DomainUpstream { ews?: NonNullable<DomainBackend['ews']> | null; version?: 2; backends?: DomainBackend[]; failover?: boolean; imap?: DomainEndpoint | null; pop3?: DomainEndpoint | null; smtp?: DomainEndpoint | null; sieve?: DomainEndpoint | null; jmap?: { sessionUrl: string } | null; dav?: { caldavUrl?: string; carddavUrl?: string } | null }
export interface ManagedDomain { aliases?: string[]; selection?: { mail?: DomainBackend | null; submission?: DomainBackend | null; rules?: DomainBackend | null } | null; name: string; enabled: boolean; preferred: boolean; upstream: DomainUpstream; settings: Record<string, unknown>; allowlist: string[]; blocklist: string[]; updatedAt: number }
export interface DomainDetail extends ManagedDomain { mode: 'engine' | 'proxy'; aliases: string[]; hosts: string[]; etag: string | null; mappingEtag: string; preferenceEtag: string }
export interface DomainChange { upstream?: DomainUpstream; settings?: Record<string, unknown>; enabled?: boolean; allowlist?: string[]; blocklist?: string[]; dryRun?: boolean }
export interface DomainPreview { dryRun: boolean; before: ManagedDomain | null; after: ManagedDomain }
const path = (name: string) => `/domains/${encodeURIComponent(name)}`;
export interface UpstreamProbe { protocol: string; host: string; port: number; tls: string; reachable: boolean; latencyMs: number; /** Exact time in μs (servers from 26.20). */ latencyUs?: number; tlsVersion: string | null; certNotAfter: string | null; certDaysLeft: number | null; greeting: string | null; capabilities: string[]; error: string | null }
export interface ProbeReport { domain: string; results: UpstreamProbe[]; ranAt: string; credentialsSent: false }
export interface DnsReport { domain: string; mx: { records: { preference: number; host: string }[]; error: string | null }; spf: { present: boolean; records: string[]; error: string | null }; dmarc: { present: boolean; records: string[]; error: string | null }; srv: { service: string; records: { priority: number; weight: number; port: number; host: string }[]; error: string | null }[] }
export interface DomainStatus { authentication?: { runtimeReady: boolean; protocols: Record<string, PasswordMechanism[]> }; transportSecurity?: { runtimeReady: boolean; auto: boolean; customCa: boolean; serverName: boolean; protocols: string[] }; mode: 'engine' | 'proxy'; blockedAccounts: { total: number; byDomain: { domain: string; accounts: number; seenHost: string | null }[] }; upstreamChoices: string[] }
export const domainsApi = {
  async usernamePreview(identity: string, username: UsernameFormat): Promise<{ authenticationUsername: string; credentialsSent: false }> { return (await adminRequest<{ authenticationUsername: string; credentialsSent: false }>('/domains/username-preview', { method: 'POST', body: { identity, username } })).data; },
  async status(): Promise<DomainStatus> { return (await adminRequest<DomainStatus>('/domains/status')).data; },
  async probe(domain: string): Promise<ProbeReport> { return (await adminRequest<ProbeReport>('/diag/upstream', { method: 'POST', body: { domain } })).data; },
  async dns(domain: string): Promise<DnsReport> { return (await adminRequest<DnsReport>('/diag/dns', { method: 'POST', body: { domain } })).data; },
  async detect(name: string): Promise<{ name: string; upstream: DomainUpstream; source: string; requiresOAuth: boolean; saved: false }> { return (await adminRequest<{ name: string; upstream: DomainUpstream; source: string; requiresOAuth: boolean; saved: false }>(`${path(name)}/detect`, { method: 'POST' })).data; },
  async capabilities(): Promise<string[]> { return (await adminRequest<{ capabilities: string[] }>('/me')).data.capabilities; },
  async list(): Promise<ManagedDomain[]> { return (await adminRequest<ManagedDomain[]>('/domains')).data; },
  async detail(name: string): Promise<DomainDetail> { const r = await adminRequest<Omit<DomainDetail, 'etag'>>(path(name)); return { ...r.data, etag: r.etag }; },
  async save(name: string, body: DomainChange, etag: string | null): Promise<ManagedDomain | DomainPreview> { return (await adminRequest<ManagedDomain | DomainPreview>(path(name), { method: 'PUT', body, ifMatch: etag })).data; },
  async remove(name: string, etag: string | null, force = false): Promise<void> { await adminRequest(`${path(name)}${force ? '?force=true' : ''}`, { method: 'DELETE', ifMatch: etag }); },
  async enable(name: string, enabled: boolean, etag: string | null): Promise<void> { await adminRequest(`${path(name)}/${enabled ? 'enable' : 'disable'}`, { method: 'POST', ifMatch: etag }); },
  async prefer(name: string, etag: string | null): Promise<void> { await adminRequest(`${path(name)}/preferred`, { method: 'PUT', ifMatch: etag }); },
  async clearPreferred(etag: string | null): Promise<void> { await adminRequest('/preferred-domain', { method: 'DELETE', ifMatch: etag }); },
  async mapping(name: string, kind: 'aliases' | 'hosts', value: string, remove: boolean, etag: string | null): Promise<void> { await adminRequest(`${path(name)}/${kind}/${encodeURIComponent(value)}`, { method: remove ? 'DELETE' : 'PUT', ifMatch: etag }); },
  async renameAlias(domain: string, oldName: string, newName: string, etag: string | null): Promise<void> { await adminRequest(`${path(domain)}/aliases/${encodeURIComponent(oldName)}`, {method:'PATCH',ifMatch:etag,body:{name:newName}}); },
};
export type DomainsApi = Omit<typeof domainsApi, 'usernamePreview'> & { usernamePreview?: typeof domainsApi.usernamePreview };

export function domainBackends(upstream: DomainUpstream): DomainBackend[] {
  if (upstream.version === 2) return structuredClone(upstream.backends ?? []);
  const rows: DomainBackend[] = [];
  for (const protocol of ['imap', 'pop3', 'smtp', 'sieve'] as const) { const endpoint = upstream[protocol]; if (endpoint) rows.push({ id: `legacy-${protocol}`, role: protocol === 'smtp' ? 'submission' : protocol === 'sieve' ? 'rules' : 'mail', protocol, enabled: true, preferred: false, priority: 100, endpoint: structuredClone(endpoint), username: { format: 'fullEmail' } }); }
  if (upstream.jmap) rows.push({ id: 'legacy-jmap', role: 'mail', protocol: 'jmap', enabled: true, preferred: false, priority: 100, sessionUrl: upstream.jmap.sessionUrl, username: { format: 'fullEmail' } });
  if (upstream.ews) rows.push({id:'legacy-ews',role:'mail',protocol:'ews',enabled:true,priority:100,ews:structuredClone(upstream.ews),username:{format:'fullEmail'}});
  return rows;
}
