import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export interface RoleGrant { roleId: string; domains: string[] | null }
export interface Override { capability: string; effect: 'allow' | 'deny' }
export interface AdminAccount { id: string; username: string; displayName: string; disabled: boolean; roles: RoleGrant[]; overrides: Override[]; totpEnrolled: boolean; passkeyCount?: number; etag?: string }
export interface AdminRole { id: string; name: string; builtin: boolean; capabilities: string[]; holders: number; etag?: string }
export interface AdminMe { username: string; capabilities: string[]; totpEnrolled: boolean }
export interface AdminSession { id: string; adminId: string; createdAt: string; current: boolean }
export const adminsApi = {
  async accounts() { return (await adminRequest<{ admins: AdminAccount[] }>('/admins')).data.admins; },
  async roles() { return (await adminRequest<{ roles: AdminRole[] }>('/roles')).data.roles; },
  async me() { return (await adminRequest<AdminMe>('/me')).data; },
  async sessions() { return (await adminRequest<{ sessions: AdminSession[] }>('/admin-sessions')).data.sessions; },
  async write<T = unknown>(path: string, method: 'POST' | 'PUT' | 'DELETE', body?: unknown, etag?: string) {
    const response = await adminRequest<T>(path, { method, body, ifMatch: etag ?? null });
    if (response.etag && response.data !== null && typeof response.data === 'object') Object.assign(response.data, { etag: response.etag });
    return response.data;
  },
};
export type AdminAccountsApi = typeof adminsApi;
