// Directory/GAL state slice (SPEC §13, plan §3 e7). Owns the app-level `DirectoryService`
// handle + a small reactive cache the composer's recipient fields share, so a repeated
// GAL query inside one compose session doesn't re-hit the server. Disjoint file — no
// `store.ts` collision (same discipline as the other V2/V3/V6 slices). Read-only (§13).
//
// e14 registers this slice; components (DirectorySearch / GroupExpand / ContactSecurity)
// may also be used standalone with their own service — the slice is the shared path.

import { createSignal, type Accessor } from 'solid-js';
import { DirectoryService, type Fetcher, type GalSearchPage } from '../../modules/directory/service.ts';
import type { GalEntry } from '../../modules/directory/index.ts';

export interface DirectorySlice {
  /** The shared typed client. */
  readonly service: DirectoryService;
  /** Whether a directory is configured at all (hides GAL UI when false). */
  enabled: Accessor<boolean>;
  setEnabled(on: boolean): void;
  /** Paged GAL search, memoised per (query,page) for the session. */
  searchGal(query: string, page?: number): Promise<GalSearchPage>;
  /** Expand a distribution group to its leaf members, memoised per DN. */
  expandGroup(dn: string): Promise<GalEntry[]>;
  /** Drop the session cache (e.g. on compose close). */
  clearCache(): void;
}

/** Build the directory slice over an injectable transport (mockable in tests). */
export function createDirectorySlice(fetcher?: Fetcher): DirectorySlice {
  const service = new DirectoryService(fetcher);
  const [enabled, setEnabled] = createSignal(false);
  const searchCache = new Map<string, GalSearchPage>();
  const groupCache = new Map<string, GalEntry[]>();

  async function searchGal(query: string, page = 0): Promise<GalSearchPage> {
    const key = `${page}${query.trim().toLowerCase()}`;
    const hit = searchCache.get(key);
    if (hit !== undefined) return hit;
    const res = await service.searchGal(query, page);
    searchCache.set(key, res);
    return res;
  }

  async function expandGroup(dn: string): Promise<GalEntry[]> {
    const hit = groupCache.get(dn);
    if (hit !== undefined) return hit;
    const members = await service.expandGroup(dn);
    groupCache.set(dn, members);
    return members;
  }

  function clearCache(): void {
    searchCache.clear();
    groupCache.clear();
  }

  return { service, enabled, setEnabled, searchGal, expandGroup, clearCache };
}
