// The local country database (t96): status, source, downloads, upload, rollback
// and a test lookup, under {console}/security/geoip.
import { adminRequest, AdminProblemError } from '../../screens/Admin/shell/adminApi.ts';
import { adminUrl } from '../adminPath.ts';
import { basePath } from '../basePath.ts';
import { apiFetch } from '../http.ts';

export type GeoipProviderId = 'dbip' | 'maxmind' | 'ipinfo' | 'custom';
export interface GeoipProvider {
  id: GeoipProviderId; name: string; needsKey: boolean; homepage: string | null;
  licence: string | null; licenceUrl: string | null; attribution: string | null; attributionUrl: string | null;
  updates: string | null; checksum: string;
}
export interface GeoipConfig {
  provider: GeoipProviderId; licenceKeyConfigured: boolean; customUrl: string; customChecksumUrl: string;
  customRedirectHosts: string[]; customCaConfigured: boolean; autoUpdate: boolean; intervalDays: number;
}
export interface GeoipDatabase {
  databaseType: string; buildEpoch: number; sizeBytes: number; sha256: string; networks: number | null;
  ipVersion: number; description: string | null; source: string; release?: string | null; label?: string | null;
  installedAt?: number; installedBy?: string; checksumVerified?: boolean; stale: boolean;
}
export interface GeoipStatus {
  managedBy: 'environment' | 'console';
  state: 'loaded' | 'missing' | 'invalid';
  error: string | null;
  dataDirectory: string | null;
  writable: boolean;
  database: GeoipDatabase | null;
  previous: { source: string; release?: string | null; buildEpoch?: number; sizeBytes?: number; installedAt?: number; sha256?: string } | null;
  policyActive: boolean;
  busy: boolean;
  config: GeoipConfig;
  providers: GeoipProvider[];
  schedule: { autoUpdate: boolean; lastCheckAt: number | null; lastSuccessAt: number | null; nextCheckAt: number | null; lastError: string | null; failures: number; waitingForFirstDownload: boolean };
  staleAfterDays: number;
  outcome?: { result: string; [key: string]: unknown };
}
export interface GeoipConfigWrite {
  provider: GeoipProviderId; licenceKey?: string; clearLicenceKey?: boolean; customUrl: string; customChecksumUrl: string;
  customRedirectHosts: string[]; customCaPem?: string; autoUpdate: boolean; intervalDays: number;
}
export interface GeoipLookup { ip: string; database: 'ready' | 'unavailable'; country: string | null }

export interface GeoipApi {
  read(): Promise<{ data: GeoipStatus; etag: string }>;
  save(config: GeoipConfigWrite, etag: string): Promise<{ data: GeoipStatus; etag: string }>;
  download(): Promise<GeoipStatus>;
  check(): Promise<GeoipStatus>;
  rollback(): Promise<GeoipStatus>;
  upload(file: Blob): Promise<GeoipStatus>;
  lookup(ip: string): Promise<GeoipLookup>;
}

async function post(path: string): Promise<GeoipStatus> {
  return (await adminRequest<GeoipStatus>(path, { method: 'POST' })).data;
}

export const geoipApi: GeoipApi = {
  async read() { const r = await adminRequest<GeoipStatus>('/security/geoip'); return { data: r.data, etag: r.etag ?? '' }; },
  async save(config, etag) { const r = await adminRequest<GeoipStatus>('/security/geoip/config', { method: 'PUT', body: config, ifMatch: etag }); return { data: r.data, etag: r.etag ?? '' }; },
  download: () => post('/security/geoip/download'),
  check: () => post('/security/geoip/check'),
  rollback: () => post('/security/geoip/rollback'),
  async upload(file) {
    // Raw bytes, not JSON. apiFetch confirms a step-up and resends the file.
    const res = await apiFetch(adminUrl('/security/geoip/upload', basePath()), {
      method: 'POST', credentials: 'same-origin', body: file,
      headers: { Accept: 'application/json', 'Content-Type': 'application/octet-stream' },
    });
    const text = await res.text();
    let body: unknown = null;
    try { body = text === '' ? null : JSON.parse(text); } catch { body = text; }
    if (!res.ok) throw new AdminProblemError(res.status, typeof body === 'object' && body !== null ? body as Record<string, unknown> : {}, null);
    return body as GeoipStatus;
  },
  async lookup(ip) { return (await adminRequest<GeoipLookup>('/security/geoip/lookup', { method: 'POST', body: { ip } })).data; },
};
