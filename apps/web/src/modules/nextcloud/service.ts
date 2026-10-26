// Nextcloud attach/save/share server I/O (SPEC §18.4, plan §3 e7). Talks to the
// `/api/nextcloud/*` surface (e9 fills over the OCS/WebDAV plugin, e14 mounts).
// Transport injectable so the pickers unit-test without a live server / Nextcloud.
// This client handles files and shares only; it does not imply calendar/contact synchronization.

import { withBase } from '../../api/basePath.ts';
import { apiFetch } from '../../api/http.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => apiFetch(input, { credentials: 'same-origin', ...init });

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`nextcloud request failed: ${res.status}`);
  return (await res.json()) as T;
}

/** A WebDAV directory entry (from the server-side PROPFIND). */
export interface WebDavEntry {
  readonly name: string;
  /** Full server-relative path (used as the id for attach/save/share). */
  readonly path: string;
  readonly isDir: boolean;
  readonly size: number;
  readonly modified: string | null;
  readonly contentType: string | null;
}

/** An attachment materialised from Nextcloud (ready for the composer). */
export interface AttachedFile {
  readonly name: string;
  /** The blob id the composer references (server streamed the file into a blob). */
  readonly blobId: string;
  readonly size: number;
  readonly contentType: string | null;
}

/** A created public share link (SPEC §18.4 large-attachment links). */
export interface ShareLink {
  readonly url: string;
  /** ISO date the link expires, or `null` for no expiry. */
  readonly expiresAt: string | null;
  /** Whether the link is password-protected (the password itself is never returned). */
  readonly passwordProtected: boolean;
}

/** Options for a share link (optional password + expiry — §18.4). */
export interface ShareLinkOptions {
  readonly path: string;
  readonly password?: string;
  /** ISO date (yyyy-mm-dd) or full ISO timestamp; omitted ⇒ no expiry. */
  readonly expiresAt?: string;
}

/**
 * The Nextcloud client. Endpoints (e9 to satisfy, e14 to mount):
 *   GET  /api/nextcloud/list?path=          → { entries: WebDavEntry[] }  (browse)
 *   POST /api/nextcloud/attach   {accountId,paths}    → { attachments: AttachedFile[] }
 *   POST /api/nextcloud/save     {accountId,emailId,blobId,path} → { entry: WebDavEntry }
 *   POST /api/nextcloud/share-link {accountId,path,password?,expiresAt?} → ShareLink
 */
export class NextcloudService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher, private readonly expectedAccountId?:string) {}
  forAccount(accountId:string|undefined):NextcloudService { return new NextcloudService(this.fetcher,accountId); }

  private async request<T>(path:string, body?:Record<string,unknown>):Promise<T> {
    const account=preferenceAccount();
    if(!account)throw new Error('Sign in before using Nextcloud.');
    if(this.expectedAccountId&&this.expectedAccountId!==account.id)throw new Error('The draft belongs to a different account.');
    const response=await this.fetcher(withBase(path),body===undefined?undefined:{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({...body,accountId:account.id})});
    const result=await jsonOrThrow<T>(response);
    if(!preferenceAccountIsCurrent(account))throw new Error('The active account changed. Open the picker again.');
    return result;
  }

  /** Browse a WebDAV directory (default the account root). */
  async list(path = '/'): Promise<WebDavEntry[]> {
    const out = await this.request<{entries:WebDavEntry[]}>(`/api/nextcloud/list?path=${encodeURIComponent(path)}`);
    return out.entries;
  }

  /** Attach one or more Nextcloud files to the current draft. */
  async attach(paths: string[]): Promise<AttachedFile[]> {
    const out = await this.request<{attachments:AttachedFile[]}>('/api/nextcloud/attach',{paths});
    return out.attachments;
  }

  /** Save an existing message attachment (by blob id) to a Nextcloud folder. */
  async saveTo(blobId: string, dir: string, emailId: string): Promise<WebDavEntry> {
    const out = await this.request<{entry:WebDavEntry}>('/api/nextcloud/save',{blobId,path:dir,emailId});
    return out.entry;
  }

  /** Create a public share link for a Nextcloud file (optional password/expiry). */
  async createShareLink(options: ShareLinkOptions): Promise<ShareLink> {
    return this.request<ShareLink>('/api/nextcloud/share-link',{...options});
  }
}
