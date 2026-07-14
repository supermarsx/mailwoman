// V7 directory/GAL module (SPEC §13, plan §2.6 / §3 e7). SCAFFOLD stub (e0):
// inert, lazily importable, typecheck-green, NOT routed. e7 fills GAL search in
// every recipient field, group expand-before-send, and the per-contact security
// tab (cert/key/verified rows); e14 wires it to /api/directory/*.

/** A resolved GAL entry (mirrors `mw_directory::GalEntry`). */
export interface GalEntry {
  readonly dn: string;
  readonly displayName: string;
  readonly mail: string;
  readonly isGroup: boolean;
}

export { DirectorySearch } from './DirectorySearch.tsx';
