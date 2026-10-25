import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface DirectoryEntry {
  id: string; url: string; baseDn: string; bindDn: string | null;
  tls: 'none' | 'starttls' | 'ldaps'; priority: number; enabled: boolean;
  attrMap: { display_name?: string | null; mail?: string | null; member?: string | null; user_cert?: string | null; photo?: string | null };
  connectionMode: 'direct' | 'blocked'; allowPrivate: boolean;
  password: { set: boolean }; revision: string;
}
export type DirectorySecret = { action: 'keep' } | { action: 'clear' } | { action: 'set'; value: string };
export type DirectoryWrite = Omit<DirectoryEntry, 'password' | 'revision'> & { password: DirectorySecret };
export interface DirectoryConfiguration {
  entries: DirectoryEntry[]; etag: string; activeRevision: string | null;
  restartRequired: boolean; reload: 'restart'; transportStatus: 'proxyUnsupported' | 'directExplicitOnly';
  environment: { bindDn: string | null; passwordSet: boolean; pinned: boolean };
}
export const directoryApi = {
  async load(): Promise<DirectoryConfiguration> { return (await adminRequest<DirectoryConfiguration>('/integrations/directory')).data; },
  async save(entries: DirectoryWrite[], etag: string): Promise<void> { await adminRequest('/integrations/directory', { method: 'PUT', body: { entries }, ifMatch: etag }); },
};
