// Nextcloud attach/save/share server I/O (SPEC §18.4, plan §3 e7). Talks to the
// `/api/nextcloud/*` surface (e9 fills over the OCS/WebDAV plugin, e14 mounts).
// Transport injectable so the pickers unit-test without a live server / Nextcloud.
// This client handles files and shares only; it does not imply calendar/contact synchronization.

import { withBase } from '../../api/basePath.ts';
import { apiFetch } from '../../api/http.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => apiFetch(input, { credentials: 'same-origin', ...init });

export class NextcloudRequestError extends Error {
  constructor(readonly status:number,readonly code:string){super(`Nextcloud request refused (${status}).`);this.name='NextcloudRequestError';}
}
async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) {const problem=await res.json().catch(()=>null) as {code?:unknown}|null;throw new NextcloudRequestError(res.status,typeof problem?.code==='string'?problem.code:'requestFailed');}
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

/** A local file stored in the linked Nextcloud (the large-attachment path). */
export interface UploadedFile {
  /** Path in the account's files, e.g. `/Mail attachments/report.zip`. */
  readonly path: string;
  readonly name: string;
  readonly size: number;
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

  private instanceId:string|undefined;
  private selectionEpoch=0;
  selectInstance(id:string):void { if(this.instanceId!==id){this.instanceId=id;this.selectionEpoch++;} }
  async instances():Promise<{id:string;name:string}[]> {
    const account=await this.request<{accountId:string;instances:{id:string;name:string}[];links:{instanceId:string;ready:boolean}[]}>('/api/account/nextcloud');
    if(account.accountId!==preferenceAccount()?.id)throw new Error('The active account changed. Sign in again.');
    return account.instances.filter(instance=>account.links.some(link=>link.instanceId===instance.id&&link.ready));
  }
  private async request<T>(path:string, body?:Record<string,unknown>):Promise<T> {
    const account=preferenceAccount();
    if(!account)throw new Error('Sign in before using Nextcloud.');
    if(this.expectedAccountId&&this.expectedAccountId!==account.id)throw new Error('The draft belongs to a different account.');
    const epoch=this.selectionEpoch;
    const target=body===undefined&&path.startsWith('/api/nextcloud/')?`${path}${path.includes('?')?'&':'?'}accountId=${encodeURIComponent(account.id)}`:path;
    const response=await this.fetcher(withBase(target),body===undefined?undefined:{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({...body,accountId:account.id,...(this.instanceId?{instanceId:this.instanceId}:{})})});
    const result=await jsonOrThrow<T>(response);
    if(epoch!==this.selectionEpoch)throw new Error('The selected Nextcloud instance changed. Try again.');
    if(!preferenceAccountIsCurrent(account))throw new Error('The active account changed. Open the picker again.');
    return result;
  }

  /** Browse a WebDAV directory (default the account root). */
  async list(path = '/'): Promise<WebDavEntry[]> {
    const out = await this.request<{entries:WebDavEntry[]}>(this.instanceId?`/api/nextcloud/files?instanceId=${encodeURIComponent(this.instanceId)}&path=${encodeURIComponent(path)}`:`/api/nextcloud/list?path=${encodeURIComponent(path)}`);
    return out.entries;
  }

  /** Attach one or more Nextcloud files to the current draft. */
  async attach(paths: string[]): Promise<AttachedFile[]> {
    const out = await this.request<{attachments:AttachedFile[]}>('/api/nextcloud/attach',{paths});
    return out.attachments;
  }

  /** Save an existing message attachment (by blob id) to a Nextcloud folder. */
  async saveTo(blobId: string, dir: string, emailId: string, acknowledgeWarning=false): Promise<WebDavEntry> {
    const out = await this.request<{entry:WebDavEntry}>('/api/nextcloud/save',{blobId,path:dir,emailId,...(acknowledgeWarning?{acknowledgeWarning:true}:{})});
    return out.entry;
  }

  /** Create a public share link for a Nextcloud file (optional password/expiry). */
  async createShareLink(options: ShareLinkOptions): Promise<ShareLink> {
    return this.request<ShareLink>('/api/nextcloud/share-link',{...options});
  }

  /** Store a local file in the linked account's "Mail attachments" folder, in
   *  chunks (the server forwards each one; it never holds the whole file). A
   *  failed or aborted upload drops the partial chunks. `onProgress` gets the
   *  bytes sent so far. */
  async uploadFile(file: Blob & {name: string}, onProgress?: (sent: number) => void, signal?: AbortSignal): Promise<UploadedFile> {
    const started = await this.request<{transfer:string;path:string;chunkBytes:number}>('/api/nextcloud/upload',{name:file.name,size:file.size});
    const step = (suffix: string) => `/api/nextcloud/upload/${encodeURIComponent(started.transfer)}/${suffix}`;
    try {
      const account = preferenceAccount();
      if (!account) throw new Error('Sign in before using Nextcloud.');
      const chunk = Math.max(1, started.chunkBytes);
      let sent = 0;
      for (let index = 1; sent < file.size; index++) {
        if (signal?.aborted) throw new DOMException('Upload cancelled.', 'AbortError');
        const part = file.slice(sent, Math.min(file.size, sent + chunk));
        const query = new URLSearchParams({accountId: account.id, path: started.path, ...(this.instanceId ? {instanceId: this.instanceId} : {})});
        const response = await this.fetcher(withBase(`${step(String(index))}?${query.toString()}`), {method: 'PUT', headers: {'content-type': 'application/octet-stream'}, body: part, ...(signal ? {signal} : {})});
        if (!response.ok) await jsonOrThrow<unknown>(response);
        if (!preferenceAccountIsCurrent(account)) throw new Error('The active account changed. Open the picker again.');
        sent += part.size;
        onProgress?.(sent);
      }
      return await this.request<UploadedFile>(step('finish'),{path:started.path,size:file.size});
    } catch (error) {
      void this.request<unknown>(step('abort'),{}).catch(() => undefined);
      throw error;
    }
  }
}
