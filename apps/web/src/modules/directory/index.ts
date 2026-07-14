// V7 directory/GAL module (SPEC §13, plan §2.6 / §3 e7). Lazily importable; NOT
// routed by this module (ownership boundary — e14 wires it into the composer's
// recipient fields + the contact card's Security tab).
//
// e14 WIRE-UP (import paths):
//   import { DirectorySearch } from './modules/directory/index.ts'  — GAL autocomplete
//                                                                     in every recipient field
//   import { GroupExpand }     from './modules/directory/index.ts'  — expand-before-send
//   import { ContactSecurity } from './modules/directory/index.ts'  — per-contact cert/key tab
// Endpoints this module calls (e9 to satisfy, e14 to mount):
//   GET /api/directory/search?q=&page=   → GalSearchPage
//   GET /api/directory/group/{dn}        → { members: GalEntry[] }
//   GET /api/directory/cert?email=       → { certs: DirectoryCert[] }
//   GET /api/directory/photo?email=      → { photoB64: string | null }

/** A resolved GAL entry (mirrors `mw_directory::GalEntry`). */
export interface GalEntry {
  readonly dn: string;
  readonly displayName: string;
  readonly mail: string;
  readonly isGroup: boolean;
}

export { DirectorySearch, type DirectorySearchProps } from './DirectorySearch.tsx';
export { GroupExpand, type GroupExpandProps } from './GroupExpand.tsx';
export { ContactSecurity, type ContactSecurityProps } from './ContactSecurity.tsx';
export { DirectoryService, type Fetcher, type GalSearchPage, type DirectoryCert } from './service.ts';
