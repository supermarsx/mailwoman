// Nextcloud attach/save/share server I/O (SPEC §18.4, plan §3 e7). Talks to the
// `/api/nextcloud/*` surface (e9 fills over the OCS/WebDAV plugin, e14 mounts).
// Transport injectable so the pickers unit-test without a live server / Nextcloud.
// CalDAV/CardDAV/tasks are core (`mw-dav`) — this module is only files + share links.

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

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
 *   POST /api/nextcloud/attach   {paths}    → { attachments: AttachedFile[] }
 *   POST /api/nextcloud/save     {blobId,attachmentId,path} → { entry: WebDavEntry }
 *   POST /api/nextcloud/share-link {path,password?,expiresAt?} → ShareLink
 */
export class NextcloudService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  /** Browse a WebDAV directory (default the account root). */
  async list(path = '/'): Promise<WebDavEntry[]> {
    const res = await this.fetcher(`/api/nextcloud/list?path=${encodeURIComponent(path)}`);
    const out = await jsonOrThrow<{ entries: WebDavEntry[] }>(res);
    return out.entries;
  }

  /** Attach one or more Nextcloud files to the current draft. */
  async attach(paths: string[]): Promise<AttachedFile[]> {
    const res = await this.fetcher('/api/nextcloud/attach', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ paths }),
    });
    const out = await jsonOrThrow<{ attachments: AttachedFile[] }>(res);
    return out.attachments;
  }

  /** Save an existing message attachment (by blob id) to a Nextcloud folder. */
  async saveTo(blobId: string, dir: string, name: string): Promise<WebDavEntry> {
    const res = await this.fetcher('/api/nextcloud/save', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ blobId, path: dir, name }),
    });
    const out = await jsonOrThrow<{ entry: WebDavEntry }>(res);
    return out.entry;
  }

  /** Create a public share link for a Nextcloud file (optional password/expiry). */
  async createShareLink(options: ShareLinkOptions): Promise<ShareLink> {
    const res = await this.fetcher('/api/nextcloud/share-link', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(options),
    });
    return jsonOrThrow<ShareLink>(res);
  }
}
