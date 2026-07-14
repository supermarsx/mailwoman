// Directory/GAL server I/O (SPEC §13, plan §2.6 / §3 e7). Talks to the `/api/directory/*`
// surface e9 fills + e14 mounts over `mw-directory`. The transport is injectable
// (`Fetcher`) so components + slices unit-test without a live server; the default uses
// same-origin cookie auth like the other slices. Directory is READ-ONLY at 1.0 (§13).

import type { GalEntry } from './index.ts';

export type Fetcher = (input: string, init?: RequestInit) => Promise<Response>;

const defaultFetcher: Fetcher = (input, init) => fetch(input, { credentials: 'same-origin', ...init });

async function jsonOrThrow<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`directory request failed: ${res.status}`);
  return (await res.json()) as T;
}

/** A page of GAL results (`GET /api/directory/search`). */
export interface GalSearchPage {
  readonly entries: GalEntry[];
  /** 0-based page index this response covers. */
  readonly page: number;
  /** Whether a further page exists (drives "load more" / paged autocomplete). */
  readonly hasMore: boolean;
}

/** An S/MIME certificate row from the directory (`GET /api/directory/cert`). */
export interface DirectoryCert {
  /** DER bytes, base64 — feeds the existing `mw-crypto` cert path (no crypto here). */
  readonly derB64: string;
  readonly fingerprint: string;
  readonly notAfter: string | null;
}

/**
 * The directory client backing GAL search, group expand-before-send, and the
 * per-contact security tab. Every method maps to exactly one `/api/directory/*`
 * endpoint (the contract e9 satisfies + e14 mounts):
 *   GET /api/directory/search?q=&page=      → GalSearchPage
 *   GET /api/directory/group/{dn}           → { members: GalEntry[] }
 *   GET /api/directory/cert?email=          → { certs: DirectoryCert[] }
 *   GET /api/directory/photo?email=         → { photoB64: string | null }
 */
export class DirectoryService {
  constructor(private readonly fetcher: Fetcher = defaultFetcher) {}

  /** Paged GAL search for every recipient field (0-based `page`). */
  async searchGal(query: string, page = 0): Promise<GalSearchPage> {
    const q = query.trim();
    if (q === '') return { entries: [], page, hasMore: false };
    const res = await this.fetcher(`/api/directory/search?q=${encodeURIComponent(q)}&page=${page}`);
    return jsonOrThrow<GalSearchPage>(res);
  }

  /**
   * Expand a distribution group to its actual leaf members ("who is actually in
   * this?") BEFORE the message is sent — the recursive flatten happens server-side
   * (`mw-directory::expand_group`); groups are excluded from the returned leaves.
   */
  async expandGroup(dn: string): Promise<GalEntry[]> {
    const res = await this.fetcher(`/api/directory/group/${encodeURIComponent(dn)}`);
    const out = await jsonOrThrow<{ members: GalEntry[] }>(res);
    return out.members;
  }

  /** S/MIME certificates published for an address (security tab). */
  async lookupCert(email: string): Promise<DirectoryCert[]> {
    const res = await this.fetcher(`/api/directory/cert?email=${encodeURIComponent(email)}`);
    const out = await jsonOrThrow<{ certs: DirectoryCert[] }>(res);
    return out.certs;
  }

  /** The directory photo for an address, base64 PNG/JPEG, or `null` (security tab). */
  async lookupPhoto(email: string): Promise<string | null> {
    const res = await this.fetcher(`/api/directory/photo?email=${encodeURIComponent(email)}`);
    const out = await jsonOrThrow<{ photoB64: string | null }>(res);
    return out.photoB64;
  }
}
