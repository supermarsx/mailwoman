import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface IdentityClientConfig { redirectUris: string[]; postLogoutRedirectUris: string[]; scopes: string[]; authMethod: 'none' | 'client_secret_basic' | 'client_secret_post' }
export interface IdentityClient { clientId: string; name: string; config: IdentityClientConfig; enabled: boolean; revision: string; secretConfigured: boolean }
export interface IdentityProvider { issuer: string; enabled: boolean; revision: string; discoveryUrl?: string; activeKeyId?: string|null }
export const identityProviderApi = {
 async provider(): Promise<IdentityProvider> { return (await adminRequest<IdentityProvider>('/oidc')).data; },
 async saveProvider(value: IdentityProvider): Promise<IdentityProvider> { return (await adminRequest<IdentityProvider>('/oidc', { method: 'PUT', ifMatch: `"${value.revision}"`, body: { issuer: value.issuer, enabled: value.enabled } })).data; },
 async rotate(key: string): Promise<void> { await adminRequest('/oidc/keys/rotate', { method: 'POST', ifMatch: `"${key}"` }); },
 async clients(): Promise<IdentityClient[]> { return (await adminRequest<{ clients: IdentityClient[] }>('/oidc/clients')).data.clients; },
 async client(id: string): Promise<IdentityClient> { return (await adminRequest<IdentityClient>(`/oidc/clients/${encodeURIComponent(id)}`)).data; },
 async save(value: { clientId?: string; revision?: string; name: string; config: IdentityClientConfig; enabled: boolean; secretAction: 'keep'|'rotate'|'clear' }): Promise<{ clientId: string; revision: string; clientSecret?: string|null }> { return (await adminRequest<{ clientId: string; revision: string; clientSecret?: string|null }>(value.clientId ? `/oidc/clients/${encodeURIComponent(value.clientId)}` : '/oidc/clients', { method: value.clientId ? 'PUT' : 'POST', ...(value.revision ? { ifMatch: `"${value.revision}"` } : {}), body: { name: value.name, config: value.config, enabled: value.enabled, secretAction: value.secretAction } })).data; },
 async remove(value: IdentityClient): Promise<void> { await adminRequest(`/oidc/clients/${encodeURIComponent(value.clientId)}`, { method: 'DELETE', ifMatch: `"${value.revision}"` }); },
};
