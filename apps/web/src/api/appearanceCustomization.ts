import { storageAuthorityHeaders } from './storageAuthority.ts';
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';
import { adminUrl } from './adminPath.ts';

export interface AppearanceDocument {version: 1; css: string; script: string}
export interface AppearanceCustomization {
  ownerId: string;
  storageAuthority?: string;
  document: AppearanceDocument;
  compiledCss: string;
  cssEnabled: boolean;
  scriptEnabled: boolean;
  cssAllowed: boolean;
  lockedFields: string[];
  scopeClass: 'mw-custom-mail' | 'mw-custom-admin';
}
export type AppearanceOwner = {kind: 'mailbox'; accountId: string} | {kind: 'admin'; username: string};
export interface CustomizationSnapshot {value: AppearanceCustomization; etag: string}
export class CustomizationConflict extends Error {
  constructor() { super('These settings changed elsewhere. Reload them before saving; your draft is still here.'); }
}
function endpoint(owner: AppearanceOwner): string {
  return owner.kind === 'mailbox'
    ? withBase(`/api/account/appearance/customization?accountId=${encodeURIComponent(owner.accountId)}`)
    : adminUrl(`/me/appearance-customization?username=${encodeURIComponent(owner.username)}`);
}
async function snapshot(response: Response): Promise<CustomizationSnapshot> {
  if (response.status === 412) throw new CustomizationConflict();
  if (response.status === 409) throw new Error('The signed-in account changed. Reload this page before saving.');
  if (!response.ok) throw new Error(response.status === 422 ? 'The customization contains unsupported CSS or exceeds its size limit.' : 'Appearance customization could not be loaded or saved.');
  const value = await response.json() as AppearanceCustomization;
  const etag = response.headers.get('etag');
  if (!etag || value.document?.version !== 1 || typeof value.ownerId !== 'string' || typeof value.compiledCss !== 'string' || !Array.isArray(value.lockedFields)) throw new Error('Invalid appearance customization response.');
  return {value, etag};
}
export async function loadCustomization(owner: AppearanceOwner, signal: AbortSignal): Promise<CustomizationSnapshot> {
  const headers = owner.kind === 'mailbox' ? storageAuthorityHeaders() : {};
  const result = await snapshot(await apiFetch(endpoint(owner), {signal, headers}));
  if (owner.kind === 'mailbox' && result.value.storageAuthority !== headers['x-mailwoman-storage-authority']) throw new Error('The signed-in account changed.');
  return result;
}
export async function saveCustomization(owner: AppearanceOwner, previous: CustomizationSnapshot, document: AppearanceDocument, signal: AbortSignal): Promise<CustomizationSnapshot> {
  return snapshot(await apiFetch(endpoint(owner), {method: 'PUT', signal, headers: {...(owner.kind === 'mailbox' ? storageAuthorityHeaders(previous.value.storageAuthority ?? '') : {}), 'content-type': 'application/json', 'if-match': previous.etag}, body: JSON.stringify({ownerId: previous.value.ownerId, document})}));
}
